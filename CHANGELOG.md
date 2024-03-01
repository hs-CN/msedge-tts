# 0.2.0
1. add `http/https`/`socks4/socks4a`/`socks5/socks5h` proxy support.
2. use `connect()`/`connect_async()`/`connect_proxy()`/`connect_proxy_async()` to create tts client now.
3. add `msedge_tts_split_proxy()`/`msedge_tts_split_proxy_async()` to create tts stream with proxy.
4. remove `ssl-key-log` feature. `SSLKEYLOGFILE` log can not be used.
# 0.2.1
fix socks5 server  authentication(`Username/Password (0x02)`) response check bug.