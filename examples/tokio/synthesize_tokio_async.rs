use msedge_tts::{
    tts::{SpeechConfig, client::tokio_runtime::connect_async},
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
                    let mut tts = connect_async().await.unwrap();
                    let start = Instant::now();
                    let audio = tts
                        .synthesize("Hello, World! 你好，世界！", &config)
                        .await
                        .unwrap();
                    println!("{:?}", audio.audio_metadata);
                    println!("{:?}", Instant::now() - start);
                    break;
                }
            }
        });
}
