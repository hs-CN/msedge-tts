use futures_util::{AsyncRead, AsyncWrite};
use msedge_tts::{
    tts::{
        client::{connect_proxy_async, MSEdgeTTSClientAsync},
        SpeechConfig,
    },
    voice::get_voices_list_async,
};
use std::time::Instant;

fn main() {
    smol::block_on(async {
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

                let tts =
                    connect_proxy_async("socks4://localhost:10808".parse().unwrap(), None, None)
                        .await
                        .unwrap();
                synthesize(tts, &config).await;

                let tts =
                    connect_proxy_async("socks5://localhost:10808".parse().unwrap(), None, None)
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
