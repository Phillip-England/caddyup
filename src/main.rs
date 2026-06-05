mod app;
mod caddyfile;
mod model;
mod terminal;
mod tui;

use anyhow::Result;

fn main() -> Result<()> {
    let path = caddyfile::find_caddyfile(std::env::current_dir()?)?;
    let document = caddyfile::CaddyDocument::load(path)?;
    terminal::run(document)
}
