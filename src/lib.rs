//! This library is a wrapper of **MSEdge Read aloud** function API.
//! You can use it to synthesize text to speech with many voices MS provided.
//!
//! # Features
//! + `native-tls`: use native tls for https and websocket. Default
//! + `ssl-key-log`: enbale `SSLKEYLOGFILE` log for some traffic analysis tools like wireshark. Debug Only
//!
//! # How to use
//! 1. You need get a [SpeechConfig](tts::SpeechConfig) to configure the voice of text to speech.  
//! You can convert [Voice](voice::Voice) to [SpeechConfig](tts::SpeechConfig) simply.  
//! Use [get_voices_list](voice) function to get all available voices.
//! You can also use [get_voices_list_async](voice) function to get all available voices asynchronously.
//! [Voice](voice::Voice) implemented [serde::Serialize] and [serde::Deserialize].  
//! For example:
//!     ```rust
//!     use msedge_tts::voice::get_voices_list;
//!     use msedge_tts::tts::SpeechConfig;
//!     
//!     fn main() {
//!         let voices = get_voices_list().unwrap();
//!         let speechConfig = SpeechConfig::from(&voices[0]);
//!     }
//!     ```
//!     You can also create [SpeechConfig](tts::SpeechConfig) by yourself. Make sure you know the right **voice name** and **audio format**.
//!
//! 2. Create a synthesis Client or Stream. Both of them have sync and async version. Example below step 3.
//!
//! 3. Synthesize text to speech.
//!     ### Sync Client
//!     Call client function [synthesize](tts::client::MSEdgeTTSClient::synthesize) to synthesize text to speech. This function return Type [SynthesizedAudio](tts::client::SynthesizedAudio),
//!     you can get [audio_bytes](tts::client::SynthesizedAudio::audio_bytes) and [audio_metadata](tts::client::SynthesizedAudio::audio_metadata).
//!     ```rust
//!     use msedge_tts::{tts::client::MSEdgeTTSClient, tts::SpeechConfig, voice::get_voices_list};
//!     
//!     fn main() {
//!         let voices = get_voices_list().unwrap();
//!         for voice in &voices {
//!             if voice.name.contains("YunyangNeural") {
//!                 let config = SpeechConfig::from(voice);
//!                 let mut tts = MSEdgeTTSClient::connect().unwrap();
//!                 let audio = tts
//!                     .synthesize("Hello, World! 你好，世界！", &config)
//!                     .unwrap();
//!                 break;
//!             }
//!         }
//!     }
//!     ```
//!     ### Async Client
//!     Call client function [synthesize_async](tts::client::MSEdgeTTSClientAsync::synthesize_async) to synthesize text to speech. This function return Type [SynthesizedAudio](tts::client::SynthesizedAudio),
//!     you can get [audio_bytes](tts::client::SynthesizedAudio::audio_bytes) and [audio_metadata](tts::client::SynthesizedAudio::audio_metadata).
//!     ```rust
//!     use msedge_tts::{tts::client::MSEdgeTTSClientAsync, tts::SpeechConfig, voice::get_voices_list_async};
//!     
//!     fn main() {
//!         smol::block_on(async {
//!             let voices = get_voices_list_async().await.unwrap();
//!             for voice in &voices {
//!                 if voice.name.contains("YunyangNeural") {
//!                     let config = SpeechConfig::from(voice);
//!                     let mut tts = MSEdgeTTSClientAsync::connect_async().await.unwrap();
//!                     let audio = tts
//!                         .synthesize_async("Hello, World! 你好，世界！", &config)
//!                         .await
//!                         .unwrap();
//!                     break;
//!                 }
//!             }
//!         });
//!     }
//!     ```
//!     ### Sync Stream
//!     Call Sender Stream function [send](tts::stream::Sender::send) to synthesize text to speech. Call Reader Stream function [read](tts::stream::Reader::read) to get data.  
//!     [read](tts::stream::Reader::read) return [Option\<SynthesizedResponse\>](tts::stream::SynthesizedResponse), the response may be [AudioBytes](tts::stream::SynthesizedResponse::AudioBytes)
//!     or [AudioMetadata](tts::stream::SynthesizedResponse::AudioMetadata) or None. This is because the **MSEdge Read aloud** API returns multiple data segment and metadata and other information sequentially.  
//!
//!     **Caution**: One [send](tts::stream::Sender::send) corresponds to multiple [read](tts::stream::Reader::read). Next [send](tts::stream::Sender::send) call will block until there no data to read.
//!     [read](tts::stream::Reader::read) will block before you call a [send](tts::stream::Sender::send).
//!     ```rust
//!     use msedge_tts::{
//!         tts::stream::{msedge_tts_split, SynthesizedResponse},
//!         tts::SpeechConfig,
//!         voice::get_voices_list,
//!     };
//!     use std::{
//!         sync::{
//!             atomic::{AtomicBool, Ordering},
//!             Arc,
//!         },
//!         thread::spawn,
//!     };
//!     
//!     fn main() {
//!         let voices = get_voices_list().unwrap();
//!         for voice in &voices {
//!             if voice.name.contains("YunyangNeural") {
//!                 let config = SpeechConfig::from(voice);
//!                 let (mut sender, mut reader) = msedge_tts_split().unwrap();
//!     
//!                 let signal = Arc::new(AtomicBool::new(false));
//!                 let end = signal.clone();
//!                 spawn(move || {
//!                     sender.send("Hello, World! 你好，世界！", &config).unwrap();
//!                     println!("synthesizing...1");
//!                     sender.send("Hello, World! 你好，世界！", &config).unwrap();
//!                     println!("synthesizing...2");
//!                     sender.send("Hello, World! 你好，世界！", &config).unwrap();
//!                     println!("synthesizing...3");
//!                     sender.send("Hello, World! 你好，世界！", &config).unwrap();
//!                     println!("synthesizing...4");
//!                     end.store(true, Ordering::Relaxed);
//!                 });
//!     
//!                 loop {
//!                     if signal.load(Ordering::Relaxed) && !reader.can_read() {
//!                         break;
//!                     }
//!                     let audio = reader.read().unwrap();
//!                     if let Some(audio) = audio {
//!                         match audio {
//!                             SynthesizedResponse::AudioBytes(_) => {
//!                                 println!("read bytes")
//!                             }
//!                             SynthesizedResponse::AudioMetadata(_) => {
//!                                 println!("read metadata")
//!                             }
//!                         }
//!                     } else {
//!                         println!("read None");
//!                     }
//!                 }
//!             }
//!         }
//!     }
//!     ```
//!     ### Async Stream
//!     Call Sender Async function [send](tts::stream::SenderAsync::send) to synthesize text to speech. Call Reader Async function [read](tts::stream::ReaderAsync::read) to get data.
//!     [read](tts::stream::ReaderAsync::read) return [Option\<SynthesizedResponse\>](tts::stream::SynthesizedResponse) as above.
//!     [send](tts::stream::SenderAsync::send) and [read](tts::stream::ReaderAsync::read) block as above.
//!     ```rust
//!     use msedge_tts::{
//!         tts::{
//!             stream::{msedge_tts_split_async, SynthesizedResponse},
//!             SpeechConfig,
//!         },
//!         voice::get_voices_list_async,
//!     };
//!     use std::{
//!         sync::{
//!             atomic::{AtomicBool, Ordering},
//!             Arc,
//!         },
//!     };
//!     
//!     fn main() {
//!         smol::block_on(async {
//!             let voices = get_voices_list_async().await.unwrap();
//!             for voice in &voices {
//!                 if voice.name.contains("YunyangNeural") {
//!                     let config = SpeechConfig::from(voice);
//!                     let (mut sender, mut reader) = msedge_tts_split_async().await.unwrap();
//!     
//!                     let signal = Arc::new(AtomicBool::new(false));
//!                     let end = signal.clone();
//!                     smol::spawn(async move {
//!                         sender
//!                             .send("Hello, World! 你好，世界！", &config)
//!                             .await
//!                             .unwrap();
//!                         println!("synthesizing...1");
//!                         sender
//!                             .send("Hello, World! 你好，世界！", &config)
//!                             .await
//!                             .unwrap();
//!                         println!("synthesizing...2");
//!                         sender
//!                             .send("Hello, World! 你好，世界！", &config)
//!                             .await
//!                             .unwrap();
//!                         println!("synthesizing...3");
//!                         sender
//!                             .send("Hello, World! 你好，世界！", &config)
//!                             .await
//!                             .unwrap();
//!                         println!("synthesizing...4");
//!                         end.store(true, Ordering::Relaxed);
//!                     })
//!                     .detach();
//!     
//!                     loop {
//!                         if signal.load(Ordering::Relaxed) && !reader.can_read().await {
//!                             break;
//!                         }
//!                         let audio = reader.read().await.unwrap();
//!                         if let Some(audio) = audio {
//!                             match audio {
//!                                 SynthesizedResponse::AudioBytes(_) => {
//!                                     println!("read bytes")
//!                                 }
//!                                 SynthesizedResponse::AudioMetadata(_) => {
//!                                     println!("read metadata")
//!                                 }
//!                             }
//!                         } else {
//!                             println!("read None");
//!                         }
//!                     }
//!                 }
//!             }
//!         });
//!     }
//!     ```

mod constants;

pub mod error;
pub mod tts;
pub mod voice;
