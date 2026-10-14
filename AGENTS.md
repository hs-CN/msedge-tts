# msedge-tts — agent instructions

Rust wrapper around the **MSEdge Read aloud** TTS API (WebSocket + HTML/SSML).

## Build & verify

```bash
cargo fmt --check
cargo clippy --all-features -- --deny warnings   # plain `cargo clippy` is NOT enough — CI denies warnings
cargo check  --all-features
cargo build  --all-features
cargo test   --all-features                       # 5 proxy unit tests + 3 lib doctests
```

Run all of these before calling a change done. CI (`.github/workflows/rust.yml`) runs them on push/PR to `master`: `fmt` + `clippy` on ubuntu; `check` + `build` + `test` on ubuntu/windows/macos. Android cross-compile lives in `.github/workflows/android.yml` and is manual (`workflow_dispatch`) only.

- **`--all-features` matters for `test`**: the only unit tests are gated behind the `proxy` feature (`src/tts/proxy/mod.rs`), so plain `cargo test` runs zero of them.
- **No network/integration tests exist.** Examples are the only end-to-end verification path; run one (below) after touching request construction or the token logic.
- Linux builds need `libasound2-dev` for the `rodio` dev-dependency (CI installs it).

## Feature flags

| Feature | Purpose |
|---|---|
| `blocking` (default) | Sync client/stream + voice list (`ureq` + `tungstenite`) |
| `smol-runtime` | Async with smol (`reqwest` + `async-tungstenite`) |
| `tokio-runtime` | Async with tokio |
| `proxy` | SOCKS4/5 + HTTP CONNECT (pairs with any runtime; not standalone) |

## Run examples

```bash
cargo run --example play                                  # blocking; quickest end-to-end smoke test (needs audio device)
cargo run --example synthesize                            # blocking, writes audio
cargo run --example play_smol_async  --features smol-runtime
cargo run --example play_tokio_async --features tokio-runtime
cargo run --example synthesize_proxy --features proxy
cargo run --example synthesize_smol_async  --features smol-runtime
cargo run --example synthesize_tokio_async --features tokio-runtime
```

Blocking examples are in `examples/` root; async examples are split into `examples/smol/` and `examples/tokio/`. Examples that need a feature declare it via `[[example]] required-features` in `Cargo.toml` — if an example won't build, check whether you passed its feature.

## Notable quirks

- **edition 2024** — needs a very recent Rust toolchain.
- **`Sec-MS-GEC` token**: Microsoft requires the **v2** token since Oct 2026; the old v1 token (plain SHA-256) is rejected with HTTP `403` during the WS handshake. Implementation is a port of upstream edge-tts in `src/tts/drm.rs` (`generate_sec_ms_gec`), selected by `SEC_MS_GEC_VERSION` in `src/constants.rs`. Only the `2-` prefix is validated; the version suffix is ignored. See `hs-CN/msedge-tts#12`, `rany2/edge-tts#490` / `#492`. The algorithm is non-standard — do not replace it with a generic SHA-256.
- **Proxy implementation** is hand-rolled (SOCKS4/5 + HTTP CONNECT) in `src/tts/proxy/`; the USERINFO parser there is the only unit-tested code.
- **`rust-analyzer`** is configured for `--all-features` (`.vscode/settings.json`).

## Project structure

- `src/`: `tts/` (`client/`, `stream/`, `proxy/`, `drm.rs`, `mod.rs`), `voice/`, `constants.rs`, `error.rs`, `lib.rs`. Each area splits into `blocking.rs` / `smol_runtime.rs` / `tokio_runtime.rs`, selected by feature flags.
- `constants.rs` holds the endpoint URLs, `USER_AGENT`, `ORIGIN`, and `SEC_MS_GEC_VERSION`.
- `examples/` as described above.
