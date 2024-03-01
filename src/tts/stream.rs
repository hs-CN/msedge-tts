//! TTS Stream module

use super::{
    super::error::Result,
    build_config_message, build_ssml_message, process_message,
    proxy::{ProxyAsyncStream, ProxyStream},
    tls::{
        websocket_connect, websocket_connect_async, websocket_connect_proxy,
        websocket_connect_proxy_async, WebSocketStream, WebSocketStreamAsync,
    },
    AudioMetadata, ProcessedMessage, SpeechConfig,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
    AsyncRead, AsyncWrite, SinkExt, StreamExt,
};
use std::{
    io::{Read, Write},
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

/// Synthesized Stream Response
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

/// Create Sync TTS Stream [Sender] and [Reader]
pub fn msedge_tts_split() -> Result<(Sender<std::net::TcpStream>, Reader<std::net::TcpStream>)> {
    _msedge_tts_split(websocket_connect()?)
}

/// Create Sync TTS Stream [Sender] and [Reader] with proxy
///
/// The proxy protocol is specified by the URI scheme.
///
/// `http`: Proxy. Default when no scheme is specified.  
/// `https`: HTTPS Proxy.  
/// `socks4`: SOCKS4 Proxy.  
/// `socks4a`: SOCKS4a Proxy. Proxy resolves URL hostname.  
/// `socks5`: SOCKS5 Proxy.  
/// `socks5h`: SOCKS5 Proxy. Proxy resolves URL hostname.  
pub fn msedge_tts_split_proxy(
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<(Sender<ProxyStream>, Reader<ProxyStream>)> {
    _msedge_tts_split(websocket_connect_proxy(proxy, username, password)?)
}

fn _msedge_tts_split<T: Read + Write>(
    websocket: WebSocketStream<T>,
) -> Result<(Sender<T>, Reader<T>)> {
    let websocket = Arc::new(Mutex::new(websocket));
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

/// Sync TTS Stream Sender
pub struct Sender<T: Read + Write> {
    websocket: Arc<Mutex<WebSocketStream<T>>>,
    can_read_cvar: Arc<(Mutex<bool>, Condvar)>,
}

impl<T: Read + Write> Sender<T> {
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

/// Sync TTS Stream Reader
pub struct Reader<T: Read + Write> {
    websocket: Arc<Mutex<WebSocketStream<T>>>,
    can_read_cvar: Arc<(Mutex<bool>, Condvar)>,
    turn_start: bool,
    response: bool,
    turn_end: bool,
}

impl<T: Read + Write> Reader<T> {
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

/// Create Async TTS Stream [SenderAsync] and [ReaderAsync]
pub async fn msedge_tts_split_async() -> Result<(
    SenderAsync<async_std::net::TcpStream>,
    ReaderAsync<async_std::net::TcpStream>,
)> {
    _msedge_tts_split_async(websocket_connect_async().await?)
}

/// Create Async TTS Stream [SenderAsync] and [ReaderAsync] with proxy
///
/// The proxy protocol is specified by the URI scheme.
///
/// `http`: Proxy. Default when no scheme is specified.  
/// `https`: HTTPS Proxy.  
/// `socks4`: SOCKS4 Proxy.  
/// `socks4a`: SOCKS4a Proxy. Proxy resolves URL hostname.  
/// `socks5`: SOCKS5 Proxy.  
/// `socks5h`: SOCKS5 Proxy. Proxy resolves URL hostname.  
pub async fn msedge_tts_split_proxy_async(
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<(SenderAsync<ProxyAsyncStream>, ReaderAsync<ProxyAsyncStream>)> {
    _msedge_tts_split_async(websocket_connect_proxy_async(proxy, username, password).await?)
}

fn _msedge_tts_split_async<T: AsyncRead + AsyncWrite + Unpin>(
    websocket: WebSocketStreamAsync<T>,
) -> Result<(SenderAsync<T>, ReaderAsync<T>)> {
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

/// Async TTS Stream Sender
pub struct SenderAsync<T> {
    sink: SplitSink<WebSocketStreamAsync<T>, tungstenite::Message>,
    can_read: Arc<async_lock::Mutex<bool>>,
}

impl<T: AsyncRead + AsyncWrite + Unpin> SenderAsync<T> {
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

/// Async TTS Stream Reader
pub struct ReaderAsync<T> {
    stream: SplitStream<WebSocketStreamAsync<T>>,
    can_read: Arc<async_lock::Mutex<bool>>,
    turn_start: bool,
    response: bool,
    turn_end: bool,
}

impl<T: AsyncRead + AsyncWrite + Unpin> ReaderAsync<T> {
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
