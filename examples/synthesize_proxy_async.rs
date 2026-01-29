use {
    msedge_tts::{
        tts::{
            SpeechConfig,
            client::{MSEdgeTTSClientAsync, connect_proxy_async},
        },
        voice::get_voices_list_async,
    },
    std::time::Instant,
    tokio::io::{AsyncRead, AsyncWrite},
};

#[tokio::main]
async fn main() {
    println!("get voices list...");
    let voices = get_voices_list_async().await.unwrap();
    for voice in &voices {
        if voice.name.contains("YunyangNeural") {
            println!("choose '{}' to synthesize...", voice.name);
            let config = SpeechConfig::from(voice);
            let tts = connect_proxy_async("localhost:10809".parse().unwrap(), None, None)
                .await
                .unwrap();
            synthesize(tts, &config).await;

            let tts = connect_proxy_async("socks4a://localhost:10808".parse().unwrap(), None, None)
                .await
                .unwrap();
            synthesize(tts, &config).await;

            let tts = connect_proxy_async("socks5h://localhost:10808".parse().unwrap(), None, None)
                .await
                .unwrap();
            synthesize(tts, &config).await;
            break;
        }
    }
}

async fn synthesize<T: AsyncRead + AsyncWrite + Unpin>(
    mut tts: MSEdgeTTSClientAsync<T>,
    config: &SpeechConfig,
) {
    let start = Instant::now();
    let audio = tts
        .synthesize("Hello, World! 你好，世界！", &config)
        .await
        .unwrap();
    println!("{:?}", audio.audio_metadata);
    println!("{:?}", Instant::now() - start);
}
