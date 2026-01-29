use {
    msedge_tts::{
        tts::{
            SpeechConfig,
            stream::{SynthesizedResponse, msedge_tts_split_async},
        },
        voice::get_voices_list_async,
    },
    std::{
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        time::Instant,
    },
};

#[tokio::main]
async fn main() {
    println!("get voices list...");
    let voices = get_voices_list_async().await.unwrap();
    for voice in &voices {
        if voice.name.contains("YunyangNeural") {
            println!("choose '{}' to synthesize...", voice.name);
            let config = SpeechConfig::from(voice);
            let (mut sender, mut reader) = msedge_tts_split_async().await.unwrap();

            let signal = Arc::new(AtomicBool::new(false));
            let end = signal.clone();
            let start = Instant::now();
            tokio::spawn(async move {
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
            });

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
    }
}
