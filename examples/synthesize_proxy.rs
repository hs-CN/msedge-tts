use msedge_tts::{
    tts::{
        client::{connect_proxy, MSEdgeTTSClient},
        SpeechConfig,
    },
    voice::get_voices_list,
};
use std::{
    io::{Read, Write},
    time::Instant,
};

fn main() {
    println!("get voices list...");
    let voices = get_voices_list().unwrap();
    for voice in &voices {
        if voice.name.contains("YunyangNeural") {
            println!("choose '{}' to synthesize...", voice.name);
            let config = SpeechConfig::from(voice);
            let tts = connect_proxy("http://localhost:10809".parse().unwrap(), None, None).unwrap();
            synthesize(tts, &config);

            let tts =
                connect_proxy("socks4://localhost:10808".parse().unwrap(), None, None).unwrap();
            synthesize(tts, &config);

            let tts =
                connect_proxy("socks5://localhost:10808".parse().unwrap(), None, None).unwrap();
            synthesize(tts, &config);
            break;
        }
    }
}

fn synthesize<T: Read + Write>(mut tts: MSEdgeTTSClient<T>, config: &SpeechConfig) {
    let start = Instant::now();
    let audio = tts
        .synthesize("Hello, World! 你好，世界！", &config)
        .unwrap();
    println!("{:?}", audio.audio_metadata);
    println!("{:?}", Instant::now() - start);
}
