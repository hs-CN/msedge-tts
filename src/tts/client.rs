//! TTS Client module

use {
    super::{
        AudioMetadata, ProcessedMessage, SpeechConfig, WebSocketStream, WebSocketStreamAsync,
        build_config_message, build_ssml_message, process_message,
        proxy::{ProxyAsyncStream, ProxyStream},
        websocket_connect, websocket_connect_async, websocket_connect_proxy,
        websocket_connect_proxy_async,
    },
    crate::error::Result,
    futures_util::{SinkExt, StreamExt},
    http::Uri,
    std::io::{Read, Write},
    tokio::{
        io::{AsyncRead, AsyncWrite},
        net::TcpStream,
    },
};

/// Sync Client
pub struct MSEdgeTTSClient<T: Read + Write>(WebSocketStream<T>);

impl<T: Read + Write> MSEdgeTTSClient<T> {
    /// Synthesize text to speech with a [SpeechConfig] synchronously
    pub fn synthesize(&mut self, text: &str, config: &SpeechConfig) -> Result<SynthesizedAudio> {
        let config_message = build_config_message(config);
        let ssml_message = build_ssml_message(text, config);
        self.0.send(config_message)?;
        self.0.send(ssml_message)?;

        let mut audio_bytes = Vec::new();
        let mut audio_metadata = Vec::new();
        let mut turn_start = false;
        let mut response = false;
        let mut turn_end = false;
        loop {
            if turn_end {
                break;
            }

            let message = self.0.read()?;
            let message = process_message(message, &mut turn_start, &mut response, &mut turn_end)?;
            if let Some(message) = message {
                match message {
                    ProcessedMessage::AudioBytes(payload) => {
                        audio_bytes.push(payload);
                    }
                    ProcessedMessage::AudioMetadata(metadata) => {
                        audio_metadata.extend(metadata);
                    }
                }
            }
        }

        let audio_bytes = audio_bytes
            .iter()
            .flat_map(|(bytes, index)| &bytes[*index..])
            .copied()
            .collect();

        Ok(SynthesizedAudio {
            audio_format: config.audio_format.clone(),
            audio_bytes,
            audio_metadata,
        })
    }
}

/// Async Client
pub struct MSEdgeTTSClientAsync<T>(WebSocketStreamAsync<T>);

impl<T: AsyncRead + AsyncWrite + Unpin> MSEdgeTTSClientAsync<T> {
    /// Synthesize text to speech with a [SpeechConfig] asynchronously
    pub async fn synthesize(
        &mut self,
        text: &str,
        config: &SpeechConfig,
    ) -> Result<SynthesizedAudio> {
        let config_message = build_config_message(config);
        let ssml_message = build_ssml_message(text, config);
        self.0.send(config_message).await?;
        self.0.send(ssml_message).await?;

        let mut audio_bytes = Vec::new();
        let mut audio_metadata = Vec::new();
        let mut turn_start = false;
        let mut response = false;
        let mut turn_end = false;
        loop {
            if turn_end {
                break;
            }

            if let Some(message) = self.0.next().await {
                let message = message?;
                let response =
                    process_message(message, &mut turn_start, &mut response, &mut turn_end)?;
                if let Some(response) = response {
                    match response {
                        ProcessedMessage::AudioBytes(payload) => {
                            audio_bytes.push(payload);
                        }
                        ProcessedMessage::AudioMetadata(metadata) => {
                            audio_metadata.extend(metadata);
                        }
                    }
                }
            }
        }

        let audio_bytes = audio_bytes
            .iter()
            .flat_map(|(bytes, index)| &bytes[*index..])
            .copied()
            .collect();

        Ok(SynthesizedAudio {
            audio_format: config.audio_format.clone(),
            audio_bytes,
            audio_metadata,
        })
    }
}

/// Synthesized Audio and Metadata
#[derive(Debug)]
pub struct SynthesizedAudio {
    pub audio_format: String,
    pub audio_bytes: Vec<u8>,
    pub audio_metadata: Vec<AudioMetadata>,
}

/// Create Sync TTS [Client](MSEdgeTTSClient)
pub fn connect() -> Result<MSEdgeTTSClient<std::net::TcpStream>> {
    Ok(MSEdgeTTSClient(websocket_connect()?))
}

/// Create Sync TTS [Client](MSEdgeTTSClient) with proxy
///
/// The proxy protocol is specified by the URI scheme.
///
/// `http`: Proxy. Default when no scheme is specified.  
/// `https`: HTTPS Proxy.  
/// `socks4`: SOCKS4 Proxy.  
/// `socks4a`: SOCKS4a Proxy. Proxy resolves URL hostname.  
/// `socks5`: SOCKS5 Proxy.  
/// `socks5h`: SOCKS5 Proxy. Proxy resolves URL hostname.  
pub fn connect_proxy(
    proxy: Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<MSEdgeTTSClient<ProxyStream>> {
    Ok(MSEdgeTTSClient(websocket_connect_proxy(
        proxy, username, password,
    )?))
}

/// Create Async TTS [Client](MSEdgeTTSClientAsync)
pub async fn connect_async() -> Result<MSEdgeTTSClientAsync<TcpStream>> {
    Ok(MSEdgeTTSClientAsync(websocket_connect_async().await?))
}

/// Create Async TTS [Client](MSEdgeTTSClientAsync) with proxy
///
/// The proxy protocol is specified by the URI scheme.
///
/// `http`: Proxy. Default when no scheme is specified.  
/// `https`: HTTPS Proxy.  
/// `socks4`: SOCKS4 Proxy.  
/// `socks4a`: SOCKS4a Proxy. Proxy resolves URL hostname.  
/// `socks5`: SOCKS5 Proxy.  
/// `socks5h`: SOCKS5 Proxy. Proxy resolves URL hostname.  
pub async fn connect_proxy_async(
    proxy: Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<MSEdgeTTSClientAsync<ProxyAsyncStream>> {
    Ok(MSEdgeTTSClientAsync(
        websocket_connect_proxy_async(proxy, username, password).await?,
    ))
}
