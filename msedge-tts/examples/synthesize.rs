use msedge_tts::{voice::get_voices_list, MSEdgeTTS, SpeechConfig};

fn main() {
    let voices = get_voices_list().unwrap();
    for voice in &voices {
        if voice.name.contains("YunyangNeural") {
            let mut tts = MSEdgeTTS::connect().unwrap();
            let audio = tts
                .synthesize("Hello, World! 你好，世界！", &SpeechConfig::from(voice))
                .unwrap();
            println!("{:?}", audio.audio_metadata);
            break;
        }
    }
}
