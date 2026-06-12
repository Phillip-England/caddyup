use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, anyhow, bail};

const RATE_LIMIT_MODULE: &str = "github.com/mholt/caddy-ratelimit@latest";
const RATE_LIMIT_MODULE_ID: &str = "http.handlers.rate_limit";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InstallOptions {
    pub bin_dir: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallReport {
    pub caddy_path: PathBuf,
    pub path_contains_bin_dir: bool,
}

pub fn install_caddy(options: InstallOptions) -> Result<InstallReport> {
    require_command("go", "Go is required to build Caddy with plugins")?;
    require_command("git", "Git is required for Go module downloads")?;

    let bin_dir = match options.bin_dir {
        Some(path) => expand_home(path)?,
        None => default_bin_dir()?,
    };
    fs::create_dir_all(&bin_dir)
        .with_context(|| format!("failed to create {}", bin_dir.display()))?;
    ensure_writable_dir(&bin_dir)?;

    let xcaddy = install_xcaddy()?;
    let caddy_path = bin_dir.join(binary_name("caddy"));
    let staged_caddy = unique_temp_path(&binary_name("caddyup-caddy-build"));

    let status = Command::new(&xcaddy)
        .arg("build")
        .arg("--with")
        .arg(RATE_LIMIT_MODULE)
        .arg("--output")
        .arg(&staged_caddy)
        .status()
        .with_context(|| format!("failed to run {}", xcaddy.display()))?;
    if !status.success() {
        bail!("xcaddy failed to build Caddy with {RATE_LIMIT_MODULE}");
    }

    fs::copy(&staged_caddy, &caddy_path)
        .with_context(|| format!("failed to install built Caddy to {}", caddy_path.display()))?;
    let _ = fs::remove_file(&staged_caddy);
    make_executable(&caddy_path)?;
    verify_rate_limit_module(&caddy_path)?;

    Ok(InstallReport {
        path_contains_bin_dir: path_contains_dir(&bin_dir),
        caddy_path,
    })
}

fn install_xcaddy() -> Result<PathBuf> {
    let status = Command::new("go")
        .arg("install")
        .arg("github.com/caddyserver/xcaddy/cmd/xcaddy@latest")
        .status()
        .context("failed to run go install for xcaddy")?;
    if !status.success() {
        bail!("go install github.com/caddyserver/xcaddy/cmd/xcaddy@latest failed");
    }

    let go_bin = go_bin_dir()?;
    let xcaddy = go_bin.join(binary_name("xcaddy"));
    if !xcaddy.exists() {
        bail!(
            "xcaddy was installed, but {} does not exist",
            xcaddy.display()
        );
    }
    Ok(xcaddy)
}

fn verify_rate_limit_module(caddy_path: &Path) -> Result<()> {
    let output = Command::new(caddy_path)
        .arg("list-modules")
        .arg("--versions")
        .output()
        .with_context(|| format!("failed to run {}", caddy_path.display()))?;
    if !output.status.success() {
        bail!("installed Caddy could not list modules");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains(RATE_LIMIT_MODULE_ID) {
        bail!("installed Caddy is missing {RATE_LIMIT_MODULE_ID}");
    }

    Ok(())
}

fn default_bin_dir() -> Result<PathBuf> {
    if let Some(path_dir) = writable_path_dir() {
        return Ok(path_dir);
    }

    home_dir()
        .map(|home| home.join(".local").join("bin"))
        .ok_or_else(|| anyhow!("could not find a writable PATH directory and HOME is not set"))
}

fn writable_path_dir() -> Option<PathBuf> {
    path_dirs()
        .into_iter()
        .filter(|path| is_user_owned_path(path))
        .find(|path| path.is_dir() && is_writable_dir(path))
}

fn is_user_owned_path(path: &Path) -> bool {
    home_dir().is_some_and(|home| path.starts_with(home))
}

fn path_dirs() -> Vec<PathBuf> {
    env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).collect())
        .unwrap_or_default()
}

fn path_contains_dir(dir: &Path) -> bool {
    path_dirs().iter().any(|candidate| candidate == dir)
}

fn go_bin_dir() -> Result<PathBuf> {
    let gobin = go_env("GOBIN")?;
    if !gobin.trim().is_empty() {
        return Ok(PathBuf::from(gobin.trim()));
    }

    let gopath = go_env("GOPATH")?;
    if !gopath.trim().is_empty() {
        return Ok(PathBuf::from(gopath.trim()).join("bin"));
    }

    home_dir()
        .map(|home| home.join("go").join("bin"))
        .ok_or_else(|| anyhow!("could not determine Go binary directory"))
}

fn go_env(name: &str) -> Result<String> {
    let output = Command::new("go")
        .arg("env")
        .arg(name)
        .output()
        .with_context(|| format!("failed to run go env {name}"))?;
    if !output.status.success() {
        bail!("go env {name} failed");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn require_command(command: &str, message: &str) -> Result<()> {
    let result = Command::new(command).arg("version").output();
    match result {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(anyhow!(
            "{message}; install `{command}` and run this command again"
        )),
    }
}

fn expand_home(path: PathBuf) -> Result<PathBuf> {
    let Some(path_string) = path.to_str() else {
        return Ok(path);
    };
    if path_string == "~" {
        return home_dir().ok_or_else(|| anyhow!("HOME is not set"));
    }
    if let Some(rest) = path_string.strip_prefix("~/") {
        return home_dir()
            .map(|home| home.join(rest))
            .ok_or_else(|| anyhow!("HOME is not set"));
    }
    Ok(path)
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

fn ensure_writable_dir(path: &Path) -> Result<()> {
    if is_writable_dir(path) {
        Ok(())
    } else {
        bail!("{} is not writable by the current user", path.display())
    }
}

fn is_writable_dir(path: &Path) -> bool {
    let probe = path.join(format!(".caddyup-write-test-{}", std::process::id()));
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
    {
        Ok(_) => {
            let _ = fs::remove_file(probe);
            true
        }
        Err(_) => false,
    }
}

fn unique_temp_path(name: &str) -> PathBuf {
    env::temp_dir().join(format!("{name}-{}", std::process::id()))
}

fn binary_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_home_dir() {
        let home = home_dir().unwrap();

        assert_eq!(
            expand_home(PathBuf::from("~/bin")).unwrap(),
            home.join("bin")
        );
    }

    #[test]
    fn binary_name_adds_windows_extension_only_on_windows() {
        let name = binary_name("caddy");

        if cfg!(windows) {
            assert_eq!(name, "caddy.exe");
        } else {
            assert_eq!(name, "caddy");
        }
    }
}
