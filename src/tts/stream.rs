//! Synthesis Stream

use super::{
    super::error::Result, build_config_message, build_ssml_message, process_message,
    tls::WebSocketStreamAsync, websocket_connect, websocket_connect_asnyc, AudioMetadata,
    ProcessedMessage, SpeechConfig, WebSocketStream,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::{
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

/// Synthesis Stream Response
#[derive(Debug)]
pub enum SynthesizedResponse {
    /// Synthesized Audio bytes segment
    AudioBytes(Vec<u8>),
    /// Synthesized Audio Metadata segment
    AudioMetadata(Vec<AudioMetadata>),
}

impl From<ProcessedMessage> for SynthesizedResponse {
    fn from(message: ProcessedMessage) -> Self {
        match message {
            ProcessedMessage::AudioBytes((bytes, index)) => {
                SynthesizedResponse::AudioBytes(bytes[index..].to_vec())
            }
            ProcessedMessage::AudioMetadata(metadata) => {
                SynthesizedResponse::AudioMetadata(metadata)
            }
        }
    }
}

/// Create Sync Synthesis Stream [Sender] and [Reader]
pub fn msedge_tts_split() -> Result<(Sender, Reader)> {
    let websocket = Arc::new(Mutex::new(websocket_connect()?));
    let can_read_cvar = Arc::new((Mutex::new(false), Condvar::new()));
    let sender = Sender {
        websocket: websocket.clone(),
        can_read_cvar: can_read_cvar.clone(),
    };
    let reader = Reader {
        websocket,
        can_read_cvar,
        turn_start: false,
        response: false,
        turn_end: false,
    };
    Ok((sender, reader))
}

/// Sync Synthesis Stream Sender
pub struct Sender {
    websocket: Arc<Mutex<WebSocketStream>>,
    can_read_cvar: Arc<(Mutex<bool>, Condvar)>,
}

impl Sender {
    /// Synthesize text to speech with a [SpeechConfig] synchronously.  
    /// **Caution**: One [send](Self::send) corresponds to multiple [read](Reader::read). Next [send](Self::send) call will block until there no data to read.
    /// [read](Reader::read) will block before you call a [send](Self::send).
    pub fn send(&mut self, text: &str, config: &SpeechConfig) -> Result<()> {
        let (can_read, cvar) = &*self.can_read_cvar;
        let mut can_read = can_read.lock().unwrap();
        while *can_read {
            can_read = cvar.wait(can_read).unwrap();
        }

        let config_message = build_config_message(config);
        let ssml_message = build_ssml_message(text, config);
        let mut websocket = self.websocket.lock().unwrap();
        websocket.send(config_message)?;
        websocket.send(ssml_message)?;

        *can_read = true;
        cvar.notify_one();
        Ok(())
    }

    /// Check if can send
    pub fn can_send(&self) -> bool {
        let (can_read, _) = &*self.can_read_cvar;
        !*can_read.lock().unwrap()
    }
}

/// Sync Synthesis Stream Reader
pub struct Reader {
    websocket: Arc<Mutex<WebSocketStream>>,
    can_read_cvar: Arc<(Mutex<bool>, Condvar)>,
    turn_start: bool,
    response: bool,
    turn_end: bool,
}

impl Reader {
    /// Read Synthesized Audio synchronously.  
    /// **Caution**: One [send](Sender::send) corresponds to multiple [read](Self::read). Next [send](Sender::send) call will block until there no data to read.
    /// [read](Self::read) will block before you call a [send](Sender::send).
    pub fn read(&mut self) -> Result<Option<SynthesizedResponse>> {
        let (can_read, cvar) = &*self.can_read_cvar;
        let mut can_read = can_read.lock().unwrap();
        while !*can_read {
            can_read = cvar.wait(can_read).unwrap();
        }

        let mut websocket = self.websocket.lock().unwrap();
        let message = process_message(
            websocket.read()?,
            &mut self.turn_start,
            &mut self.response,
            &mut self.turn_end,
        )?;

        if self.turn_start && self.response && self.turn_end {
            self.turn_start = false;
            self.response = false;
            self.turn_end = false;
            *can_read = false;
            cvar.notify_one();
        }

        Ok(message.map(|message| message.into()))
    }

    /// Check if can read
    pub fn can_read(&self) -> bool {
        let (can_read, _) = &*self.can_read_cvar;
        *can_read.lock().unwrap()
    }
}

/// Create Async Synthesis Stream [SenderAsync] and [ReaderAsync]
pub async fn msedge_tts_split_async() -> Result<(SenderAsync, ReaderAsync)> {
    let websocket = websocket_connect_asnyc().await?;
    let (sink, stream) = websocket.split();
    let can_read = Arc::new(async_lock::Mutex::new(false));
    Ok((
        SenderAsync {
            sink,
            can_read: can_read.clone(),
        },
        ReaderAsync {
            stream,
            can_read,
            turn_start: false,
            response: false,
            turn_end: false,
        },
    ))
}

/// Async Synthesis Stream Sender
pub struct SenderAsync {
    sink: SplitSink<WebSocketStreamAsync, tungstenite::Message>,
    can_read: Arc<async_lock::Mutex<bool>>,
}

impl SenderAsync {
    /// Synthesize text to speech with a [SpeechConfig] asynchronously.  
    /// **Caution**: One [send](Self::send) corresponds to multiple [read](ReaderAsync::read). Next [send](Self::send) call will block until there no data to read.
    /// [read](ReaderAsync::read) will block before you call a [send](Self::send).
    pub async fn send(&mut self, text: &str, config: &SpeechConfig) -> Result<()> {
        while !self.can_send().await {
            async_io::Timer::after(Duration::from_millis(1)).await;
        }
        let mut can_read = self.can_read.lock().await;
        let config_message = build_config_message(config);
        let ssml_message = build_ssml_message(text, config);
        self.sink.send(config_message).await?;
        self.sink.send(ssml_message).await?;
        *can_read = true;
        Ok(())
    }

    /// Check if can send
    pub async fn can_send(&self) -> bool {
        !*self.can_read.lock().await
    }
}

/// Async Synthesis Stream Reader
pub struct ReaderAsync {
    stream: SplitStream<WebSocketStreamAsync>,
    can_read: Arc<async_lock::Mutex<bool>>,
    turn_start: bool,
    response: bool,
    turn_end: bool,
}

impl ReaderAsync {
    /// Read Synthesized Audio asynchronously.  
    /// **Caution**: One [send](SenderAsync::send) corresponds to multiple [read](Self::read). Next [send](SenderAsync::send) call will block until there no data to read.
    /// [read](Self::read) will block before you call a [send](SenderAsync::send).
    pub async fn read(&mut self) -> Result<Option<SynthesizedResponse>> {
        while !self.can_read().await {
            async_io::Timer::after(Duration::from_millis(1)).await;
        }

        let message = self.stream.next().await;
        if let Some(message) = message {
            let message = message?;
            let message = process_message(
                message,
                &mut self.turn_start,
                &mut self.response,
                &mut self.turn_end,
            )?;

            if self.turn_start && self.response && self.turn_end {
                self.turn_start = false;
                self.response = false;
                self.turn_end = false;
                *self.can_read.lock().await = false;
            }

            Ok(message.map(|message| message.into()))
        } else {
            Ok(None)
        }
    }

    /// Check if can read
    pub async fn can_read(&self) -> bool {
        *self.can_read.lock().await
    }
}
