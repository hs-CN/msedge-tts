# msedge-tts — agent instructions

## Build & verify

```bash
cargo fmt --check
cargo clippy --all-features
cargo check --all-features
cargo build --all-features
cargo test                   # no tests exist — only examples
```

CI runs these on push/PR to `master` via `.github/workflows/rust.yml`. Can also trigger manually with `workflow_dispatch`.

Platform coverage: `fmt` + `clippy` on ubuntu, `build` + `test` on ubuntu/windows/macos, `android` cross‑compile via `cross` (manual trigger only, see `.github/workflows/android.yml`).

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

## Project structure

- Library code in `src/`: `tts/` (client, stream, proxy), `voice/`, `error.rs`, `constants.rs`, `lib.rs`
- Examples in `examples/` with runtime-specific subdirectories: `blocking/`, `smol/`, `tokio/`
- Each example demonstrates different feature combinations (blocking, smol-runtime, tokio-runtime, with/without proxy)