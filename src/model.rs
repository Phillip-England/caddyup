#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Site {
    pub address: String,
    pub upstream: String,
    pub rate_limit_enabled: bool,
    pub rate_limit_zone: String,
    pub rate_limit_events: u32,
    pub rate_limit_window: String,
    pub encode_zstd: bool,
    pub encode_gzip: bool,
    pub tls_internal: bool,
}

impl Default for Site {
    fn default() -> Self {
        Self {
            address: "example.com".to_string(),
            upstream: "127.0.0.1:3000".to_string(),
            rate_limit_enabled: false,
            rate_limit_zone: "dynamic".to_string(),
            rate_limit_events: 100,
            rate_limit_window: "1m".to_string(),
            encode_zstd: true,
            encode_gzip: true,
            tls_internal: false,
        }
    }
}

impl Site {
    pub fn title(&self) -> String {
        format!("{} -> {}", self.address, self.upstream)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CaddyConfig {
    pub sites: Vec<Site>,
}

impl CaddyConfig {
    pub fn with_example_site() -> Self {
        Self {
            sites: vec![Site::default()],
        }
    }
}
