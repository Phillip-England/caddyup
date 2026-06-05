# caddyup

`caddyup` is a Rust TUI for managing common Caddyfile server blocks without hand-editing the file.

Run it from the directory that contains your `Caddyfile`:

```sh
cargo run
```

If no `Caddyfile` exists in the current directory, caddyup starts with a new managed config and writes `./Caddyfile` when you save.

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

The first version supports the high-frequency Caddyfile cases: server address, reverse proxy upstream, compression, internal TLS, and app-level rate-limit settings. Rate limiting is emitted with a note because it requires a Caddy build that includes a compatible rate-limit module.
