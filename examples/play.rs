use msedge_tts::{
    tts::{client::connect, SpeechConfig},
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
            let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
            let sink = stream_handle
                .play_once(std::io::Cursor::new(audio.audio_bytes))
                .unwrap();
            sink.sleep_until_end();
            println!("play audio done.");

            break;
        }
    }
}
