# caddyup

`caddyup` is a Rust TUI for managing common Caddyfile server blocks without hand-editing the file.

Run it from the directory that contains your `Caddyfile`:

```sh
cargo run
```

If no `Caddyfile` exists in the current directory, caddyup starts with a new managed config and writes `./Caddyfile` when you save.

## Installing Caddy

caddyup can install a Caddy binary that already includes the rate-limit module used by generated configs:

```sh
caddyup install-caddy
```

The installer uses Go and `xcaddy` to build Caddy with `github.com/mholt/caddy-ratelimit`, then places the resulting `caddy` binary in a user-writable bin directory. It prefers an existing user-owned directory on `PATH` and falls back to `~/.local/bin`.

To choose the install location explicitly:

```sh
caddyup install-caddy --bin-dir ~/.local/bin
```

After installation, caddyup verifies that the installed Caddy binary exposes `http.handlers.rate_limit`.

## Controls

- `up/down` or `j/k`: move between fields
- `left/right` or `h/l`: move between servers
- `enter` or `space`: edit a text field or toggle a boolean field
- `a`: add server
- `d`: delete server
- `s`: save
- `q` or `esc`: quit

## Caddyfile Strategy

caddyup preserves any existing Caddyfile content outside this managed block:

```caddyfile
# caddyup:begin
# generated server blocks
# caddyup:end
```

The first version supports the high-frequency Caddyfile cases: server address, reverse proxy upstream, compression, internal TLS, and app-level rate-limit settings. Use `caddyup install-caddy` to install a Caddy binary with the needed rate-limit module.
