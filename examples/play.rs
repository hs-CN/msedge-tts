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
            let stream_handle = rodio::OutputStreamBuilder::open_default_stream().unwrap();
            let sink = rodio::Sink::connect_new(&stream_handle.mixer());

            let decoder =
                rodio::decoder::Decoder::new(std::io::Cursor::new(audio.audio_bytes)).unwrap();

            sink.append(decoder);
            sink.sleep_until_end();
            println!("play audio done.");

            break;
        }
    }
}
