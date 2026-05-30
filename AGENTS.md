# msedge-tts — agent instructions

## Build & verify

```bash
cargo fmt --check
cargo clippy --all-features
cargo check --all-features
cargo build --all-features
cargo test                   # no #[test] functions — examples-only verification
```

CI runs these on push/PR to `master` via `.github/workflows/rust.yml`. Platform coverage: `fmt` + `clippy` on ubuntu; `check` + `build` + `test` on ubuntu/windows/macos. Android cross-compile via `cross` in `.github/workflows/android.yml` (manual `workflow_dispatch` only).

## Feature flags

| Feature | Purpose |
|---|---|
| `blocking` (default) | Sync client/stream + voice list |
| `smol-runtime` | Async with smol |
| `tokio-runtime` | Async with tokio |
| `proxy` | SOCKS4/5 + HTTP CONNECT (pairs with any runtime) |

`proxy` is not standalone — pair it with a runtime feature.

## Run examples

```bash
cargo run --example synthesize                          # blocking (default)
cargo run --example synthesize_proxy --features proxy    # blocking + proxy
cargo run --example synthesize_smol_async --features smol-runtime
cargo run --example synthesize_tokio_async --features tokio-runtime
```

Blocking examples live in `examples/` root. Async examples are under `examples/smol/` and `examples/tokio/`. Examples requiring `proxy` declare it via `[[example]] required-features` in `Cargo.toml`.

## Notable quirks

- **edition 2024** — requires a very recent Rust toolchain.
- **No `#[test]` functions** anywhere; examples are the only verification path.
- **`rust-analyzer`** is configured for `--all-features` (`.vscode/settings.json`).
- **`Sec-MS-GEC` header** generated via SHA-256 of truncated ticks as a China 403 workaround (see `gen_sec_ms_gec()` in `src/tts/mod.rs:166`).
- **Proxy implementation** is hand-rolled (SOCKS4/5 + HTTP CONNECT) in `src/tts/proxy/`.
- **Linux build** needs `libasound2-dev` for `rodio` dev-dependency (installed automatically in CI).

## Project structure

- Library in `src/`: `tts/` (client, stream, proxy), `voice/`, `error.rs`, `constants.rs`, `lib.rs`
- Blocking examples in `examples/`, async examples split into `examples/smol/` and `examples/tokio/`
