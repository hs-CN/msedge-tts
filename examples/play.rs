use msedge_tts::{
    tts::{SpeechConfig, client::connect},
    voice::get_voices_list,
};
use std::time::Instant;

fn main() {
    println!("get voices list...");
    let voices = get_voices_list().unwrap();
    for voice in &voices {
        if voice.name.contains("YunyangNeural") {
            println!("choose '{}' to synthesize...", voice.name);
            let config = SpeechConfig::from(voice);
            let mut tts = connect().unwrap();
            let start = Instant::now();
            let audio = tts
                .synthesize("Hello, World! 你好，世界！", &config)
                .unwrap();
            println!("{:?}", audio.audio_metadata);
            println!("{:?}", Instant::now() - start);

            println!("play audio...");
            let handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
            let player = rodio::Player::connect_new(&handle.mixer());

            let decoder =
                rodio::decoder::Decoder::new(std::io::Cursor::new(audio.audio_bytes)).unwrap();

            player.append(decoder);
            player.sleep_until_end();
            println!("play audio done.");

            break;
        }
    }
}
