use msedge_tts::{
    tts::{
        SpeechConfig,
        stream::{
            SynthesizedResponse,
            smol_runtime::{ReceiverAsync, SenderAsync, msedge_tts_split_proxy_async},
        },
    },
    voice::smol_runtime::get_voices_list_async,
};
use smol::io::{AsyncRead, AsyncWrite};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

fn main() {
    smol::block_on(async {
        println!("get voices list...");
        let voices = get_voices_list_async().await.unwrap();
        for voice in &voices {
            if voice.name.contains("YunyangNeural") {
                println!("choose '{}' to synthesize...", voice.name);
                let config = Arc::new(SpeechConfig::from(voice));
                let (sender, reader) = msedge_tts_split_proxy_async(
                    "http://127.0.0.1:7897".parse().unwrap(),
                    None,
                    None,
                )
                .await
                .unwrap();
                synthesize(sender, reader, config.clone()).await;

                let (sender, reader) = msedge_tts_split_proxy_async(
                    "socks4a://127.0.0.1:7897".parse().unwrap(),
                    None,
                    None,
                )
                .await
                .unwrap();
                synthesize(sender, reader, config.clone()).await;

                let (sender, reader) = msedge_tts_split_proxy_async(
                    "socks5h://127.0.0.1:7897".parse().unwrap(),
                    None,
                    None,
                )
                .await
                .unwrap();
                synthesize(sender, reader, config.clone()).await;
                break;
            }
        }
    });
}

async fn synthesize<T: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static>(
    mut sender: SenderAsync<T>,
    mut reader: ReceiverAsync<T>,
    config: Arc<SpeechConfig>,
) {
    let signal = Arc::new(AtomicBool::new(false));
    let end = signal.clone();
    let start = Instant::now();
    smol::spawn(async move {
        sender
            .send("Hello, World! 你好，世界！", &config)
            .await
            .unwrap();
        println!("synthesizing...1");
        sender
            .send("Hello, World! 你好，世界！", &config)
            .await
            .unwrap();
        println!("synthesizing...2");
        sender
            .send("Hello, World! 你好，世界！", &config)
            .await
            .unwrap();
        println!("synthesizing...3");
        sender
            .send("Hello, World! 你好，世界！", &config)
            .await
            .unwrap();
        println!("synthesizing...4");
        end.store(true, Ordering::Relaxed);
    })
    .detach();

    loop {
        if signal.load(Ordering::Relaxed) && !reader.can_read().await {
            break;
        }
        let audio = reader.read().await.unwrap();
        if let Some(audio) = audio {
            match audio {
                SynthesizedResponse::AudioBytes(_) => {
                    println!("read bytes")
                }
                SynthesizedResponse::AudioMetadata(_) => {
                    println!("read metadata")
                }
            }
        } else {
            println!("read None");
        }
    }
    println!("{:?}", Instant::now() - start)
}
