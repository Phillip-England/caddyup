mod app;
mod caddyfile;
mod installer;
mod model;
mod terminal;
mod tui;

use std::{env, path::PathBuf};

use anyhow::{Result, bail};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if let Some(command) = args.first() {
        match command.as_str() {
            "install-caddy" => return install_caddy_command(&args[1..]),
            "-h" | "--help" | "help" => {
                print_help();
                return Ok(());
            }
            _ => bail!("unknown command `{command}`; run `caddyup --help`"),
        }
    }

    let path = caddyfile::find_caddyfile(std::env::current_dir()?)?;
    let document = caddyfile::CaddyDocument::load(path)?;
    terminal::run(document)
}

fn install_caddy_command(args: &[String]) -> Result<()> {
    let options = parse_install_options(args)?;
    let report = installer::install_caddy(options)?;

    println!("Installed Caddy with rate limiting:");
    println!("  {}", report.caddy_path.display());
    if report.path_contains_bin_dir {
        println!("`caddy` is available on your PATH.");
    } else if let Some(parent) = report.caddy_path.parent() {
        println!(
            "{} is not currently on PATH. Add it to PATH or run Caddy by its full path.",
            parent.display()
        );
    }

    Ok(())
}

fn parse_install_options(args: &[String]) -> Result<installer::InstallOptions> {
    let mut options = installer::InstallOptions::default();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--bin-dir" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    bail!("--bin-dir requires a path");
                };
                options.bin_dir = Some(PathBuf::from(value));
            }
            "-h" | "--help" => {
                print_install_help();
                std::process::exit(0);
            }
            arg => bail!("unknown install-caddy option `{arg}`"),
        }
        index += 1;
    }

    Ok(options)
}

fn print_help() {
    println!(
        "caddyup\n\nUsage:\n  caddyup\n  caddyup install-caddy [--bin-dir PATH]\n\nCommands:\n  install-caddy  Build and install Caddy with rate limiting support"
    );
}

fn print_install_help() {
    println!(
        "caddyup install-caddy\n\nUsage:\n  caddyup install-caddy [--bin-dir PATH]\n\nBuilds Caddy with github.com/mholt/caddy-ratelimit and installs it to a user-writable bin directory. If --bin-dir is omitted, caddyup uses a writable user-owned PATH directory or falls back to ~/.local/bin."
    );
}
