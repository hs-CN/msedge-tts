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

                    println!("play audio...");
                    let handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
                    let player = rodio::Player::connect_new(&handle.mixer());

                    let decoder =
                        rodio::decoder::Decoder::new(std::io::Cursor::new(audio.audio_bytes))
                            .unwrap();

                    player.append(decoder);
                    player.sleep_until_end();
                    println!("play audio done.");

                    break;
                }
            }
        });
}
