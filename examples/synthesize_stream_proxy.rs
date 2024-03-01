use msedge_tts::{
    tts::{
        stream::{msedge_tts_split_proxy, Reader, Sender, SynthesizedResponse},
        SpeechConfig,
    },
    voice::get_voices_list,
};
use std::{
    io::{Read, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::spawn,
    time::Instant,
};

fn main() {
    println!("get voices list...");
    let voices = get_voices_list().unwrap();
    for voice in &voices {
        if voice.name.contains("YunyangNeural") {
            println!("choose '{}' to synthesize...", voice.name);
            let config = Arc::new(SpeechConfig::from(voice));
            let (sender, reader) =
                msedge_tts_split_proxy("http://127.0.0.1:10809".parse().unwrap(), None, None)
                    .unwrap();
            synthesize(sender, reader, config.clone());

            let (sender, reader) =
                msedge_tts_split_proxy("socks4a://127.0.0.1:10808".parse().unwrap(), None, None)
                    .unwrap();
            synthesize(sender, reader, config.clone());

            let (sender, reader) =
                msedge_tts_split_proxy("socks5h://127.0.0.1:10808".parse().unwrap(), None, None)
                    .unwrap();
            synthesize(sender, reader, config.clone());
            break;
        }
    }
}

fn synthesize<T: Read + Write + Send + Sync + 'static>(
    mut sender: Sender<T>,
    mut reader: Reader<T>,
    config: Arc<SpeechConfig>,
) {
    let signal = Arc::new(AtomicBool::new(false));
    let end = signal.clone();
    let start = Instant::now();
    spawn(move || {
        sender.send("Hello, World! 你好，世界！", &config).unwrap();
        println!("synthesizing...1");
        sender.send("Hello, World! 你好，世界！", &config).unwrap();
        println!("synthesizing...2");
        sender.send("Hello, World! 你好，世界！", &config).unwrap();
        println!("synthesizing...3");
        sender.send("Hello, World! 你好，世界！", &config).unwrap();
        println!("synthesizing...4");
        end.store(true, Ordering::Relaxed);
    });

    loop {
        if signal.load(Ordering::Relaxed) && !reader.can_read() {
            break;
        }
        let audio = reader.read().unwrap();
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
    println!("{:?}", Instant::now() - start);
}
