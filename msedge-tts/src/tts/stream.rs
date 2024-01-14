use super::{
    build_config_message, build_ssml_message, process_message, tls::WebSocketStreamAsync,
    websocket_connect, websocket_connect_asnyc, AudioMetadata, ProcessedMessage, SpeechConfig,
    WebSocketStream,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug)]
pub enum SynthesizedResponse {
    AudioBytes(Vec<u8>),
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

pub fn msedge_tts_split() -> anyhow::Result<(Sender, Reader)> {
    let websocket = Arc::new(Mutex::new(websocket_connect()?));
    let idle_cvar = Arc::new((Mutex::new(true), Condvar::new()));
    let sender = Sender {
        websocket: websocket.clone(),
        idle_cvar: idle_cvar.clone(),
    };
    let reader = Reader {
        websocket,
        idle_cvar,
        turn_start: false,
        response: false,
        turn_end: false,
    };
    Ok((sender, reader))
}

pub struct Sender {
    websocket: Arc<Mutex<WebSocketStream>>,
    idle_cvar: Arc<(Mutex<bool>, Condvar)>,
}

impl Sender {
    pub fn send(&mut self, text: &str, config: &SpeechConfig) -> anyhow::Result<()> {
        let (guard, cvar) = &*self.idle_cvar;
        let mut idle = guard.lock().unwrap();
        while !*idle {
            idle = cvar.wait(idle).unwrap();
        }

        let config_message = build_config_message(config);
        let ssml_message = build_ssml_message(text, config);
        let mut websocket = self.websocket.lock().unwrap();
        websocket.send(config_message)?;
        websocket.send(ssml_message)?;

        *idle = false;
        cvar.notify_one();
        Ok(())
    }

    pub fn is_idle(&self) -> bool {
        let (guard, _) = &*self.idle_cvar;
        *guard.lock().unwrap()
    }
}

pub struct Reader {
    websocket: Arc<Mutex<WebSocketStream>>,
    idle_cvar: Arc<(Mutex<bool>, Condvar)>,
    turn_start: bool,
    response: bool,
    turn_end: bool,
}

impl Reader {
    pub fn read(&mut self) -> anyhow::Result<Option<SynthesizedResponse>> {
        let (guard, cvar) = &*self.idle_cvar;
        let mut idle = guard.lock().unwrap();
        while *idle {
            idle = cvar.wait(idle).unwrap();
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
            *idle = true;
            cvar.notify_one();
        }

        Ok(message.and_then(|message| Some(message.into())))
    }

    pub fn is_idle(&self) -> bool {
        let (guard, _) = &*self.idle_cvar;
        *guard.lock().unwrap()
    }
}

pub async fn msedge_tts_split_asnyc() -> anyhow::Result<(SenderAsync, ReaderAsync)> {
    let websocket = websocket_connect_asnyc().await?;
    let (sink, stream) = websocket.split();
    Ok((
        SenderAsync { sink },
        ReaderAsync {
            stream,
            turn_start: false,
            response: false,
            turn_end: false,
        },
    ))
}

pub struct SenderAsync {
    sink: SplitSink<WebSocketStreamAsync, tungstenite::Message>,
}

impl SenderAsync {
    pub async fn send(&mut self, text: &str, config: &SpeechConfig) -> anyhow::Result<()> {
        let config_message = build_config_message(config);
        let ssml_message = build_ssml_message(text, config);
        self.sink.send(config_message).await?;
        self.sink.send(ssml_message).await?;
        Ok(())
    }
}

pub struct ReaderAsync {
    stream: SplitStream<WebSocketStreamAsync>,
    turn_start: bool,
    response: bool,
    turn_end: bool,
}

impl ReaderAsync {
    pub async fn read(&mut self) -> anyhow::Result<Option<SynthesizedResponse>> {
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
            }

            Ok(message.and_then(|message| Some(message.into())))
        } else {
            Ok(None)
        }
    }
}
