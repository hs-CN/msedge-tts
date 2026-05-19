use std::{sync::Arc, time::Duration};

use async_tungstenite::{WebSocketReceiver, WebSocketSender, WebSocketStream, smol::ConnectStream};
use smol::{
    Timer,
    io::{AsyncRead, AsyncWrite},
    lock::Mutex,
    stream::StreamExt,
};

use crate::{
    error::Result,
    tts::{
        Payload, SpeechConfig, build_config_message, build_ssml_message,
        smol_runtime::websocket_connect_async, stream::SynthesizedResponse,
    },
};

/// Async TTS Stream Sender
pub struct SenderAsync<T> {
    sender: WebSocketSender<T>,
    can_read: Arc<Mutex<bool>>,
}

impl<T: AsyncRead + AsyncWrite + Unpin> SenderAsync<T> {
    /// Synthesize text to speech with a [SpeechConfig] asynchronously.  
    /// **Caution**: One [send](Self::send) corresponds to multiple [read](ReaderAsync::read). Next [send](Self::send) call will block until there no data to read.
    /// [read](ReaderAsync::read) will block before you call a [send](Self::send).
    pub async fn send(&mut self, text: &str, config: &SpeechConfig) -> Result<()> {
        while !self.can_send().await {
            Timer::after(Duration::from_millis(1)).await;
        }
        let mut can_read = self.can_read.lock().await;
        let config_message = build_config_message(config);
        let ssml_message = build_ssml_message(text, config);
        self.sender.send(config_message).await?;
        self.sender.send(ssml_message).await?;
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
    receiver: WebSocketReceiver<T>,
    can_read: Arc<Mutex<bool>>,
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
            Timer::after(Duration::from_millis(1)).await;
        }

        let message = self.receiver.next().await;
        if let Some(message) = message {
            let message = message?;
            let payload = Payload::process(
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

            Ok(payload.map(|payload| payload.into()))
        } else {
            Ok(None)
        }
    }

    /// Check if can read
    pub async fn can_read(&self) -> bool {
        *self.can_read.lock().await
    }
}

/// Create Async TTS Stream [SenderAsync] and [ReaderAsync]
pub async fn msedge_tts_split_async()
-> Result<(SenderAsync<ConnectStream>, ReaderAsync<ConnectStream>)> {
    split(websocket_connect_async().await?)
}

#[cfg(feature = "proxy")]
use crate::tts::{
    proxy::smol_runtime::ProxyAsyncStream, smol_runtime::websocket_connect_proxy_async,
};

/// Create Async TTS Stream [SenderAsync] and [ReaderAsync] with proxy
///
/// The proxy protocol is specified by the URI scheme.
///
/// `http`: Proxy. Default when no scheme is specified.  
/// `https`: HTTPS Proxy.  
/// `socks4`: SOCKS4 Proxy.  
/// `socks4a`: SOCKS4a Proxy. Proxy resolves URL hostname.  
/// `socks5`: SOCKS5 Proxy.  
/// `socks5h`: SOCKS5 Proxy. Proxy resolves URL hostname.  \
#[cfg(feature = "proxy")]
pub async fn msedge_tts_split_proxy_async(
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<(
    SenderAsync<async_tungstenite::smol::ClientStream<ProxyAsyncStream>>,
    ReaderAsync<async_tungstenite::smol::ClientStream<ProxyAsyncStream>>,
)> {
    split(websocket_connect_proxy_async(proxy, username, password).await?)
}

fn split<T: AsyncRead + AsyncWrite + Unpin>(
    websocket: WebSocketStream<T>,
) -> Result<(SenderAsync<T>, ReaderAsync<T>)> {
    let (sender, receiver) = websocket.split();
    let can_read = Arc::new(Mutex::new(false));
    Ok((
        SenderAsync {
            sender,
            can_read: can_read.clone(),
        },
        ReaderAsync {
            receiver,
            can_read,
            turn_start: false,
            response: false,
            turn_end: false,
        },
    ))
}
