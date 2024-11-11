//! TTS Client and Stream, SpeechConfig, Response Type.

pub mod client;
pub mod stream;

mod proxy;
use crate::error::{Error, ProxyError, Result};
use proxy::{
    http_proxy, http_proxy_async, socks4_proxy, socks4_proxy_async, socks5_proxy,
    socks5_proxy_asnyc, ProxyAsyncStream, ProxyStream,
};

use sha2::Digest;

/// Synthesis Config
#[derive(Debug)]
pub struct SpeechConfig {
    pub voice_name: String,
    /// should be one of Streaming or NonStreaming audio output formats.
    ///
    /// Streaming audio output formats:
    /// + amr-wb-16000hz
    /// + audio-16khz-16bit-32kbps-mono-opus
    /// + audio-16khz-32kbitrate-mono-mp3
    /// + audio-16khz-64kbitrate-mono-mp3
    /// + audio-16khz-128kbitrate-mono-mp3
    /// + audio-24khz-16bit-24kbps-mono-opus
    /// + audio-24khz-16bit-48kbps-mono-opus
    /// + audio-24khz-48kbitrate-mono-mp3
    /// + audio-24khz-96kbitrate-mono-mp3
    /// + audio-24khz-160kbitrate-mono-mp3
    /// + audio-48khz-96kbitrate-mono-mp3
    /// + audio-48khz-192kbitrate-mono-mp3
    /// + ogg-16khz-16bit-mono-opus
    /// + ogg-24khz-16bit-mono-opus
    /// + ogg-48khz-16bit-mono-opus
    /// + raw-8khz-8bit-mono-alaw
    /// + raw-8khz-8bit-mono-mulaw
    /// + raw-8khz-16bit-mono-pcm
    /// + raw-16khz-16bit-mono-pcm
    /// + raw-16khz-16bit-mono-truesilk
    /// + raw-22050hz-16bit-mono-pcm
    /// + raw-24khz-16bit-mono-pcm
    /// + raw-24khz-16bit-mono-truesilk
    /// + raw-44100hz-16bit-mono-pcm
    /// + raw-48khz-16bit-mono-pcm
    /// + webm-16khz-16bit-mono-opus
    /// + webm-24khz-16bit-24kbps-mono-opus
    /// + webm-24khz-16bit-mono-opus
    ///
    /// NonStreaming audio output formats:
    /// + riff-8khz-8bit-mono-alaw
    /// + riff-8khz-8bit-mono-mulaw
    /// + riff-8khz-16bit-mono-pcm
    /// + riff-22050hz-16bit-mono-pcm
    /// + riff-24khz-16bit-mono-pcm
    /// + riff-44100hz-16bit-mono-pcm
    /// + riff-48khz-16bit-mono-pcm
    pub audio_format: String,
    pub pitch: i32,
    pub rate: i32,
    pub volume: i32,
}

impl From<&super::voice::Voice> for SpeechConfig {
    fn from(voice: &super::voice::Voice) -> Self {
        let audio_output_format = if let Some(ref output_format) = voice.suggested_codec {
            output_format.clone()
        } else {
            "audio-24khz-48kbitrate-mono-mp3".to_string()
        };
        Self {
            voice_name: voice.name.clone(),
            audio_format: audio_output_format,
            pitch: 0,
            rate: 0,
            volume: 0,
        }
    }
}

/// Audio Metadata
#[derive(Debug)]
pub struct AudioMetadata {
    pub metadata_type: Option<String>,
    pub offset: u64,
    pub duration: u64,
    pub text: Option<String>,
    pub length: u64,
    pub boundary_type: Option<String>,
}

impl AudioMetadata {
    fn from_str(text: &str) -> Result<Vec<Self>> {
        let value: serde_json::Value = serde_json::from_str(text)?;
        if let Some(items) = value["Metadata"].as_array() {
            let mut audio_metadata = Vec::new();
            for item in items {
                let metadata_type = item["Type"].as_str().map(|x| x.to_owned());
                let offset = item["Data"]["Offset"].as_u64().unwrap_or(0);
                let duration = item["Data"]["Duration"].as_u64().unwrap_or(0);
                let text = item["Data"]["text"]["Text"].as_str().map(|x| x.to_owned());
                let length = item["Data"]["text"]["Length"].as_u64().unwrap_or(0);
                let boundary_type = item["Data"]["text"]["BoundaryType"]
                    .as_str()
                    .map(|x| x.to_owned());
                audio_metadata.push(AudioMetadata {
                    metadata_type,
                    offset,
                    duration,
                    text,
                    length,
                    boundary_type,
                });
            }
            Ok(audio_metadata)
        } else {
            Err(Error::UnexpectedMessage(format!(
                "unexpected json text: {}",
                text
            )))
        }
    }
}

enum ProcessedMessage {
    AudioBytes((Vec<u8>, usize)),
    AudioMetadata(Vec<AudioMetadata>),
}

fn process_message(
    message: tungstenite::Message,
    turn_start: &mut bool,
    response: &mut bool,
    turn_end: &mut bool,
) -> Result<Option<ProcessedMessage>> {
    match message {
        tungstenite::Message::Text(text) => {
            if text.contains("audio.metadata") {
                if let Some(index) = text.find("\r\n\r\n") {
                    let metadata = AudioMetadata::from_str(&text[index + 4..])?;
                    Ok(Some(ProcessedMessage::AudioMetadata(metadata)))
                } else {
                    Ok(None)
                }
            } else if text.contains("turn.start") {
                *turn_start = true;
                Ok(None)
            } else if text.contains("response") {
                *response = true;
                Ok(None)
            } else if text.contains("turn.end") {
                *turn_end = true;
                Ok(None)
            } else {
                Err(Error::UnexpectedMessage(format!(
                    "unexpected text message: {}",
                    text
                )))
            }
        }
        tungstenite::Message::Binary(bytes) => {
            if *turn_start || *response {
                let header_len = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
                Ok(Some(ProcessedMessage::AudioBytes((bytes, header_len + 2))))
            } else {
                Ok(None)
            }
        }
        tungstenite::Message::Close(_) => {
            *turn_end = true;
            Ok(None)
        }
        _ => Err(Error::UnexpectedMessage(format!(
            "unexpected message: {}",
            message
        ))),
    }
}

// try to solve china mainland 403 forbidden issue
// solution from:
// https://github.com/rany2/edge-tts/issues/290#issuecomment-2464956570
fn gen_sec_ms_gec() -> String {
    // UTC time from 1601-01-01
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        + std::time::Duration::from_secs(11644473600);
    let ticks = duration.as_nanos() / 100;
    let ticks = ticks - ticks % 3_000_000_000;

    let mut hasher = sha2::Sha256::new();
    hasher.update(format!("{ticks}6A5AA1D4EAFF4E9FB37E23D68491D6F4"));
    let hash_code = hasher.finalize();
    let mut hex_str = String::new();
    for &byte in hash_code.iter() {
        hex_str.push_str(&format!("{:02X}", byte));
    }
    hex_str
}

fn build_websocket_request() -> Result<tungstenite::handshake::client::Request> {
    use super::constants;
    use tungstenite::client::IntoClientRequest;
    use tungstenite::http::header;

    let uuid = uuid::Uuid::new_v4().simple().to_string();
    let sec_ms_gec = gen_sec_ms_gec();
    let sec_ms_gec_version = "1-130.0.2849.68";
    let mut request = format!(
        "{}{}&Sec-MS-GEC={}&Sec-MS-GEC-Version={}",
        constants::WSS_URL,
        uuid,
        sec_ms_gec,
        sec_ms_gec_version
    )
    .into_client_request()?;
    let headers = request.headers_mut();
    headers.insert(
        header::PRAGMA,
        "no-cache"
            .parse()
            .map_err(|err| tungstenite::Error::from(http::Error::from(err)))?,
    );
    headers.insert(
        header::CACHE_CONTROL,
        "no-cache"
            .parse()
            .map_err(|err| tungstenite::Error::from(http::Error::from(err)))?,
    );
    headers.insert(
        header::USER_AGENT,
        constants::USER_AGENT
            .parse()
            .map_err(|err| tungstenite::Error::from(http::Error::from(err)))?,
    );
    headers.insert(
        header::ORIGIN,
        constants::ORIGIN
            .parse()
            .map_err(|err| tungstenite::Error::from(http::Error::from(err)))?,
    );
    Ok(request)
}

fn build_config_message(config: &SpeechConfig) -> tungstenite::Message {
    static SPEECH_CONFIG_HEAD: &str = r#"{"context":{"synthesis":{"audio":{"metadataoptions":{"sentenceBoundaryEnabled":"false","wordBoundaryEnabled":"true"},"outputFormat":""#;
    static SPEECH_CONFIG_TAIL: &str = r#""}}}}"#;
    let speech_config_message = format!(
        "X-Timestamp:{}\r\nContent-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n{}{}{}",
        chrono::Local::now().to_rfc2822(),
        SPEECH_CONFIG_HEAD,
        config.audio_format,
        SPEECH_CONFIG_TAIL
    );
    tungstenite::Message::Text(speech_config_message)
}

fn build_ssml_message(text: &str, config: &SpeechConfig) -> tungstenite::Message {
    let ssml = format!(
        "<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xml:lang='en-US'><voice name='{}'><prosody pitch='{:+}Hz' rate='{:+}%' volume='{:+}%'>{}</prosody></voice></speak>",
        config.voice_name,
        config.pitch,
        config.rate,
        config.volume,
        text,
    );
    let ssml_message = format!(
        "X-RequestId:{}\r\nContent-Type:application/ssml+xml\r\nX-Timestamp:{}\r\nPath:ssml\r\n\r\n{}",
        uuid::Uuid::new_v4().simple(),
        chrono::Local::now().to_rfc2822(),
        ssml,
    );
    tungstenite::Message::Text(ssml_message)
}

type WebSocketStream<T> = tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<T>>;

fn websocket_connect() -> Result<WebSocketStream<std::net::TcpStream>> {
    let request = build_websocket_request()?;
    let (websocket, _) = tungstenite::connect(request)?;
    Ok(websocket)
}

fn websocket_connect_proxy(
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<WebSocketStream<ProxyStream>> {
    use tungstenite::handshake::HandshakeError;

    let request = build_websocket_request()?;
    let stream: std::result::Result<ProxyStream, ProxyError> = match proxy.scheme_str() {
        Some(scheme) => match scheme.to_lowercase().as_str() {
            "socks4" | "socks4a" => {
                socks4_proxy(request.uri().host().unwrap(), proxy, username).map_err(|e| e.into())
            }
            "socks5" | "socks5h" => {
                socks5_proxy(request.uri().host().unwrap(), proxy, username, password)
                    .map_err(|e| e.into())
            }
            "http" | "https" => {
                http_proxy(request.uri().host().unwrap(), proxy, username, password)
                    .map_err(|e| e.into())
            }
            _ => Err(ProxyError::NotSupportedScheme(proxy)),
        },
        None => http_proxy(request.uri().host().unwrap(), proxy, username, password)
            .map_err(|e| e.into()),
    };
    let (websocket, _) = tungstenite::client_tls(request, stream?).map_err(|e| match e {
        HandshakeError::Failure(e) => e,
        HandshakeError::Interrupted(_) => panic!("Bug: blocking handshake not blocked"),
    })?;
    Ok(websocket)
}

type WebSocketStreamAsync<T> =
    async_tungstenite::WebSocketStream<async_tungstenite::async_std::ClientStream<T>>;

async fn websocket_connect_async() -> Result<WebSocketStreamAsync<async_std::net::TcpStream>> {
    let request = build_websocket_request()?;
    let (websocket, _) = async_tungstenite::async_std::connect_async(request).await?;
    Ok(websocket)
}

async fn websocket_connect_proxy_async(
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<WebSocketStreamAsync<ProxyAsyncStream>> {
    let request = build_websocket_request()?;
    let stream: std::result::Result<ProxyAsyncStream, ProxyError> = match proxy.scheme_str() {
        Some(scheme) => match scheme.to_lowercase().as_str() {
            "socks4" | "socks4a" => {
                socks4_proxy_async(request.uri().host().unwrap(), proxy, username)
                    .await
                    .map_err(|e| e.into())
            }
            "socks5" | "socks5h" => {
                socks5_proxy_asnyc(request.uri().host().unwrap(), proxy, username, password)
                    .await
                    .map_err(|e| e.into())
            }
            "http" | "https" => {
                http_proxy_async(request.uri().host().unwrap(), proxy, username, password)
                    .await
                    .map_err(|e| e.into())
            }
            _ => Err(ProxyError::NotSupportedScheme(proxy)),
        },
        None => http_proxy_async(request.uri().host().unwrap(), proxy, username, password)
            .await
            .map_err(|e| e.into()),
    };
    let (websocket, _) = async_tungstenite::async_std::client_async_tls(request, stream?).await?;
    Ok(websocket)
}
