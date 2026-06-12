use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use thiserror::Error;

use crate::model::{CaddyConfig, Site};

const BEGIN_MARKER: &str = "# caddyup:begin";
const END_MARKER: &str = "# caddyup:end";

#[derive(Debug, Error)]
enum CaddyfileError {
    #[error("managed caddyup block starts but does not end")]
    MissingManagedBlockEnd,
}

#[derive(Clone, Debug)]
pub struct CaddyDocument {
    pub path: PathBuf,
    prefix: String,
    suffix: String,
    pub config: CaddyConfig,
}

impl CaddyDocument {
    pub fn load(path: PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(&path).unwrap_or_default();
        let (prefix, managed, suffix) = split_managed_block(&contents)?;
        let config = managed
            .map(parse_managed_block)
            .unwrap_or_else(CaddyConfig::default);

        Ok(Self {
            path,
            prefix,
            suffix,
            config,
        })
    }

    pub fn save(&self) -> Result<()> {
        fs::write(&self.path, self.render())
            .with_context(|| format!("failed to write {}", self.path.display()))
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(self.prefix.trim_end());
        if !output.is_empty() {
            output.push_str("\n\n");
        }
        output.push_str(BEGIN_MARKER);
        output.push('\n');
        output.push_str(&render_config(&self.config));
        output.push_str(END_MARKER);
        output.push('\n');
        let suffix = self.suffix.trim_start();
        if !suffix.is_empty() {
            output.push('\n');
            output.push_str(suffix);
        }
        output
    }
}

pub fn find_caddyfile(cwd: PathBuf) -> Result<PathBuf> {
    for candidate in ["Caddyfile", "caddyfile"] {
        let path = cwd.join(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    Ok(cwd.join("Caddyfile"))
}

fn split_managed_block(contents: &str) -> Result<(String, Option<String>, String)> {
    let Some(begin) = contents.find(BEGIN_MARKER) else {
        return Ok((contents.to_string(), None, String::new()));
    };
    let after_begin = begin + BEGIN_MARKER.len();
    let Some(relative_end) = contents[after_begin..].find(END_MARKER) else {
        return Err(CaddyfileError::MissingManagedBlockEnd.into());
    };
    let end = after_begin + relative_end;
    let after_end = end + END_MARKER.len();

    Ok((
        contents[..begin].to_string(),
        Some(contents[after_begin..end].to_string()),
        contents[after_end..].to_string(),
    ))
}

fn render_config(config: &CaddyConfig) -> String {
    let mut output = String::new();
    for site in &config.sites {
        output.push_str(&format!("{} {{\n", site.address));
        if site.encode_zstd || site.encode_gzip {
            let mut encoders = Vec::new();
            if site.encode_zstd {
                encoders.push("zstd");
            }
            if site.encode_gzip {
                encoders.push("gzip");
            }
            output.push_str(&format!("    encode {}\n", encoders.join(" ")));
        }
        if site.tls_internal {
            output.push_str("    tls internal\n");
        }
        if site.rate_limit_enabled {
            output.push_str("    rate_limit {\n");
            output.push_str(&format!("        zone {} {{\n", site.rate_limit_zone));
            output.push_str(&format!(
                "            key {{remote_host}}\n            events {}\n            window {}\n",
                site.rate_limit_events, site.rate_limit_window
            ));
            output.push_str("        }\n");
            output.push_str("    }\n");
        }
        output.push_str(&format!("    reverse_proxy {}\n", site.upstream));
        output.push_str("}\n\n");
    }
    output
}

fn parse_managed_block(block: String) -> CaddyConfig {
    let mut sites = Vec::new();
    let mut current: Option<Site> = None;
    let mut in_rate_limit = false;

    for raw_line in block.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.ends_with('{') && !line.starts_with("rate_limit") && !line.starts_with("zone ") {
            if let Some(site) = current.take() {
                sites.push(site);
            }
            current = Some(Site {
                address: line.trim_end_matches('{').trim().to_string(),
                ..Site::default()
            });
            in_rate_limit = false;
            continue;
        }

        let Some(site) = current.as_mut() else {
            continue;
        };

        if line == "}" {
            if in_rate_limit {
                in_rate_limit = false;
            }
            continue;
        }

        if let Some(value) = line.strip_prefix("reverse_proxy ") {
            site.upstream = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("encode ") {
            site.encode_zstd = value.split_whitespace().any(|item| item == "zstd");
            site.encode_gzip = value.split_whitespace().any(|item| item == "gzip");
        } else if line == "tls internal" {
            site.tls_internal = true;
        } else if line.starts_with("rate_limit") {
            site.rate_limit_enabled = true;
            in_rate_limit = true;
        } else if in_rate_limit {
            if let Some(value) = line.strip_prefix("zone ") {
                site.rate_limit_zone = value
                    .trim_end_matches('{')
                    .split_whitespace()
                    .next()
                    .unwrap_or("dynamic")
                    .to_string();
            } else if let Some(value) = line.strip_prefix("events ") {
                site.rate_limit_events = value.trim().parse().unwrap_or(site.rate_limit_events);
            } else if let Some(value) = line.strip_prefix("window ") {
                site.rate_limit_window = value.trim().to_string();
            }
        }
    }

    if let Some(site) = current {
        sites.push(site);
    }

    CaddyConfig { sites }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn round_trips_managed_sites() {
        let config = CaddyConfig {
            sites: vec![Site {
                address: "api.example.com".to_string(),
                upstream: "localhost:8080".to_string(),
                rate_limit_enabled: true,
                rate_limit_events: 25,
                rate_limit_window: "30s".to_string(),
                ..Site::default()
            }],
        };

        let rendered = render_config(&config);
        let parsed = parse_managed_block(rendered);

        assert_eq!(parsed, config);
    }

    #[test]
    fn preserves_unmanaged_prefix_and_suffix() {
        let input = "admin.example.com {\n    respond ok\n}\n\n# caddyup:begin\nold.example.com {\n    reverse_proxy 127.0.0.1:5000\n}\n# caddyup:end\n\n(common) {\n    encode gzip\n}\n";
        let (prefix, managed, suffix) = split_managed_block(input).unwrap();

        assert!(prefix.contains("admin.example.com"));
        assert!(managed.unwrap().contains("old.example.com"));
        assert!(suffix.contains("(common)"));
    }

    #[test]
    fn new_file_uses_empty_config() {
        let (prefix, managed, suffix) = split_managed_block("").unwrap();

        assert!(prefix.is_empty());
        assert!(managed.is_none());
        assert!(suffix.is_empty());
    }

    #[test]
    fn find_existing_caddyfile_case_preference() {
        let path = find_caddyfile(Path::new("/tmp/example").to_path_buf()).unwrap();

        assert_eq!(path, Path::new("/tmp/example/Caddyfile"));
    }
}
