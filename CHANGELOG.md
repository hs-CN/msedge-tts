# 0.2.4
add derive `Clone` for struct `Voice` and struct `VoiceTag`;  
add derive `Clone`,`serde::Deserialize`,`serde::Serialize` for struct `SpeechConfig`;
# 0.2.3
try to fix china mainland 403 forbidden issue.
solution from:
https://github.com/rany2/edge-tts/issues/290#issuecomment-2464956570

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