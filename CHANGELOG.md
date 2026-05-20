# 0.4.0
## Breaking changes
- **Full feature flag rewrite**: Added `blocking`, `smol-runtime`, `tokio-runtime`, `proxy` features. `default = ["blocking"]`.
- **HTTP client replaced**: `isahc` → `ureq` (blocking) / `reqwest` (async).
- **TLS stack replaced**: `native-tls` / `async-native-tls` → `rustls` + `rustls-platform-verifier`.
- **Async runtime replaced**: `async-std` / `async-io` / `async-lock` → `smol` / `tokio`.
- **Proxy implementation rewritten**: Hand-rolled SOCKS4/5 + HTTP CONNECT in `src/tts/proxy/`. `ProxyStream`/`ProxyAsyncStream` enums replace old `native-tls`-based types.
- **Error type restructured**: `Error::IsahcError` → `Error::UreqError`/`Error::ReqwestError` (feature-gated). `Error::ProxyError` is now `cfg(feature = "proxy")`.
- **`Voice::from(String)` removed**.
- **`constants.rs` cleaned up**: Removed unused constants (`SEC_CH_UA`, `SEC_CH_UA_MOBILE`, `SEC_CH_UA_PLATFORM`, `SEC_FETCH_SITE`, `SEC_FETCH_MODE`, `SEC_FETCH_DEST`).

## New features
- **Feature flags**: `blocking` (default, sync), `smol-runtime` (smol-based async), `tokio-runtime` (tokio-based async), `proxy` (SOCKS4/5 + HTTP CONNECT). Pair `proxy` with any runtime feature.
- **Split stream API**: `msedge_tts_split()` / `msedge_tts_split_async()` return `(Sender, Receiver)` pairs for streaming synthesis.
- **Async dual-runtime**: Full support for both `smol` and `tokio` runtimes, selectable via feature flag.


## Dependency changes
### Added
- `ureq` 3.3.0 (optional, blocking HTTP)
- `reqwest` 0.13.3 (optional, async HTTP)
- `rustls` 0.23.40 (optional)
- `rustls-platform-verifier` 0.7.0 (optional)
- `http` 1.4.0 (optional, proxy)
- `httparse` 1.10.1 (optional, proxy)
- `base64` 0.22.1 (optional, proxy)
- `async-compat` 0.2.5 (optional, smol-runtime)
- `smol` 2.0.2 (optional)
- `futures-rustls` 0.26.0 (optional, smol-runtime)
- `futures-util` 0.3.32 (optional)
- `tokio` 1.52.3 (optional)
- `tokio-rustls` 0.26.4 (optional)
- `async-tungstenite` 0.34.1 (optional)
- `rodio` 0.22.2 (dev)

### Removed
- `isahc`
- `native-tls` / `async-native-tls`
- `async-std` / `async-io` / `async-lock`

### Updated
- `chrono` 0.4.43 → 0.4.44
- `sha2` 0.10.9 → 0.11.0
- `tungstenite` 0.28.0 → 0.29.0
- `uuid` 1.19.0 → 1.23.1
- `serde` 1.0.228 (same)
- `serde_json` 1.0.149 (same)
- `thiserror` 2.0.18 (same)

## Examples
- **Reorganized**: Split into `examples/smol/` and `examples/tokio/` directories.
- **New examples**: `get_voices_list_smol_async`, `get_voices_list_tokio_async`, `synthesize_tokio_async`, `synthesize_stream_tokio_async`, `play` and proxy variants for each runtime.
- **All examples** now declare `required-features` for proper `cargo run` filtering.

## Other
- Added `opencode.json` for IDE LSP config.
- Added `AGENTS.md` for AI agent instructions.

# 0.3.0
## rust edition update to 2024
## dependencies update:  
async-io "2.4.0" -> "2.6.0"  
async-lock "3.4.0" -> "3.4.2"  
async-std "1.13.0" -> "1.13.2"  
async-tungstenite "0.28.0" -> "0.32.1"  
chrono "0.4.38" -> "0.4.43"  
http "1.1.0" -> "1.4.0"  
httparse "1.9.5" -> "1.10.1"  
native-tls "0.2.12" -> "0.2.14"  
serde "1.0.214" -> "1.0.228"  
serde_json "1.0.132" -> "1.0.149"  
sha2 "0.10.8" -> "0.10.9"  
thiserror "2.0.3" -> "2.0.18"  
tungstenite "0.24.0" -> "0.28.0"  
uuid "1.11.0" -> "1.19.0"  
rodio "0.20.1" -> "0.21.1"  

# 0.2.5
try to fix https://github.com/hs-CN/msedge-tts/issues/8  
thanks @[icsboyx](https://github.com/icsboyx)
# 0.2.4
add derive `Clone` for struct `Voice` and struct `VoiceTag`;  
add derive `Clone`,`serde::Deserialize`,`serde::Serialize` for struct `SpeechConfig`;
# 0.2.3
try to fix china mainland 403 forbidden issue.
solution from:
https://github.com/rany2/edge-tts/issues/290#issuecomment-2464956570  
thanks @[super1207](https://github.com/super1207)

dependencies update:
> `async-io` from `2.3.4` to `2.4.0`;  
> `futures-util` from `0.3.30` to `0.3.31`;  
> `httparse` from `1.9.4` to `1.9.5`;  
> `serde` from `1.0.210` to `1.0.214`;  
> `serde_json` from `1.0.128` to `1.0.132`;  
> `thiserror` from `1.0.63` to `2.0.3`;  
> `uuid` from `1.10.0` to `1.11.0`;  
# 0.2.2
dependencies update:
> `async-io` from `2.3.1` to `2.3.4`;  
> `async-lock` from `3.3.0` to `3.4.0`;  
> `async-std` from `1.12.0` to `1.13.0`;  
> `async-tungstenite` from `0.25.0` to `0.28.0`;  
> `base64` from `0.21.7` to `0.22.1;`  
> `chrono` from `0.4.34` to `0.4.38;`  
> `http` from `1.0.0` to `1.1.0;`  
> `httparse` from `1.8.0` to `1.9.4;`  
> `native-tls` from `0.2.11` to `0.2.12`;  
> `serde` from `1.0.197` to `1.0.210;`  
> `serde_json` from `1.0.114` to `1.0.128;`  
> `thiserror` from `1.0.57` to `1.0.63;`  
> `tungstenite` from `0.21.0` to `0.24.0;`  
> `uuid` from `1.7.0` to `1.10.0;`  
> `smol` from `2.0.0` to `2.0.2;`  
# 0.2.1
fix socks5 server  authentication(`Username/Password (0x02)`) response check bug.
# 0.2.0
1. add `http/https`/`socks4/socks4a`/`socks5/socks5h` proxy support.
2. use `connect()`/`connect_async()`/`connect_proxy()`/`connect_proxy_async()` to create tts client now.
3. add `msedge_tts_split_proxy()`/`msedge_tts_split_proxy_async()` to create tts stream with proxy.
4. remove `ssl-key-log` feature. `SSLKEYLOGFILE` log can not be used.