use msedge_tts::{tts::MSEdgeTTS, tts::SpeechConfig, voice::get_voices_list};

fn main() {
    println!("get voices list...");
    let voices = get_voices_list().unwrap();
    for voice in &voices {
        if voice.name.contains("YunyangNeural") {
            println!("choose '{}' to synthesize...", voice.name);
            let mut tts = MSEdgeTTS::connect().unwrap();
            let audio = tts
                .synthesize("Hello, World! 你好，世界！", &SpeechConfig::from(voice))
                .unwrap();
            println!("{:?}", audio.audio_metadata);
            break;
        }
    }
}
