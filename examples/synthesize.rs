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
            break;
        }
    }
}
