use futures_util::io::{AsyncRead, AsyncWrite};
use msedge_tts::{
    tts::{
        SpeechConfig,
        client::{MSEdgeTTSClientAsync, tokio_runtime::connect_proxy_async},
    },
    voice::tokio_runtime::get_voices_list_async,
};
use std::time::Instant;

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            println!("get voices list...");
            let voices = get_voices_list_async().await.unwrap();
            for voice in &voices {
                if voice.name.contains("YunyangNeural") {
                    println!("choose '{}' to synthesize...", voice.name);
                    let config = SpeechConfig::from(voice);
                    let tts = connect_proxy_async("http://127.0.0.1:7897").await.unwrap();
                    synthesize(tts, &config).await;

                    let tts = connect_proxy_async("socks4a://127.0.0.1:7897")
                        .await
                        .unwrap();
                    synthesize(tts, &config).await;

                    let tts = connect_proxy_async("socks5h://127.0.0.1:7897")
                        .await
                        .unwrap();
                    synthesize(tts, &config).await;
                    break;
                }
            }
        });
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
