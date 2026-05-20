# msedge-tts — agent instructions

## Build & verify

```bash
cargo fmt --check
cargo clippy --all-features
cargo check --all-features
cargo build --all-features
cargo test                   # no tests exist — only examples
```

No CI — run the above manually.

## Feature flags

| Feature | Purpose |
|---|---|
| `blocking` (default) | Sync client/stream + voice list |
| `smol-runtime` | Async with smol |
| `tokio-runtime` | Async with tokio |
| `proxy` | SOCKS4/5 + HTTP CONNECT (augments any runtime) |

`proxy` is not standalone — pair it with a runtime feature.

## Run an example

```bash
cargo run --example synthesize                          # blocking (default)
cargo run --example synthesize_proxy --features proxy    # blocking + proxy
cargo run --example synthesize_smol_async --features smol-runtime
cargo run --example synthesize_tokio_async --features tokio-runtime
```

Examples requiring `proxy` declare it in `Cargo.toml` `[[example]]` `required-features`.

## Notable quirks

- **edition 2024** — needs a very recent Rust toolchain.
- **No `#[test]` functions** anywhere. Examples are the only verification path.
- **`rust-analyzer`** in VS Code is configured for `--all-features` (`.vscode/settings.json`).
- **`opencode.json`** only sets `"lsp": true`.
- **`Sec-MS-GEC`** header is generated via SHA-256 of ticks as a China 403 workaround (see `gen_sec_ms_gec()` in `src/tts/mod.rs`).
- **Proxy implementation** is custom (hand-rolled SOCKS4/5 + HTTP CONNECT in `src/tts/proxy/`).
