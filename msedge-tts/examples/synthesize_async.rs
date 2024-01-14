use msedge_tts::{
    tts::client::MSEdgeTTSClientAsync, tts::SpeechConfig, voice::get_voices_list_async,
};

fn main() {
    smol::block_on(async {
        println!("get voices list...");
        let voices = get_voices_list_async().await.unwrap();
        for voice in &voices {
            if voice.name.contains("YunyangNeural") {
                println!("choose '{}' to synthesize...", voice.name);
                let config = SpeechConfig::from(voice);
                let mut tts = MSEdgeTTSClientAsync::connect_async().await.unwrap();
                let audio = tts
                    .synthesize_async("Hello, World! 你好，世界！", &config)
                    .await
                    .unwrap();
                println!("{:?}", audio.audio_metadata);
                break;
            }
        }
    });
}
