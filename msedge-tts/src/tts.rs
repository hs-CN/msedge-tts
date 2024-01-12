use std::net::TcpStream;

use tungstenite::{
    client::IntoClientRequest, handshake::client::Request, http::header, stream::MaybeTlsStream,
    WebSocket,
};

use crate::{constants, voice::Voice};

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

impl From<&Voice> for SpeechConfig {
    fn from(voice: &Voice) -> Self {
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
    fn from_str(text: &str) -> anyhow::Result<Vec<Self>> {
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
            anyhow::bail!("unexpected json text: {:#?}", value)
        }
    }
}

#[derive(Debug)]
pub struct SynthesizedAudio {
    pub audio_format: String,
    pub audio_bytes: Vec<u8>,
    pub audio_metadata: Vec<AudioMetadata>,
}

pub struct MSEdgeTTS(WebSocket<MaybeTlsStream<TcpStream>>);

impl MSEdgeTTS {
    pub fn connect() -> anyhow::Result<Self> {
        Ok(Self(websocket_connect()?))
    }

    pub fn synthesize(
        &mut self,
        text: &str,
        config: &SpeechConfig,
    ) -> anyhow::Result<SynthesizedAudio> {
        let config_message = build_config_message(config);
        let ssml_message = build_ssml_message(text, config);
        self.0.send(config_message)?;
        self.0.send(ssml_message)?;
        self.0.flush()?;

        let mut audio_payload = Vec::new();
        let mut audio_metadata = Vec::new();
        let mut turn_start = false;
        let mut response = false;
        loop {
            let message = self.0.read()?;
            match message {
                tungstenite::Message::Text(text) => {
                    if text.contains("audio.metadata") {
                        if let Some(index) = text.find("\r\n\r\n") {
                            audio_metadata.extend(AudioMetadata::from_str(&text[index + 4..])?);
                        }
                    } else if text.contains("turn.start") {
                        turn_start = true;
                    } else if text.contains("response") {
                        response = true;
                    } else if text.contains("turn.end") {
                        break;
                    } else {
                        anyhow::bail!("unexpected text message: {}", text)
                    }
                }
                tungstenite::Message::Binary(bytes) => {
                    if turn_start || response {
                        let header_len = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
                        audio_payload.push((bytes, header_len + 2));
                    }
                }
                tungstenite::Message::Close(_) => break,
                _ => anyhow::bail!("unexpected message: {}", message),
            }
        }

        let audio_bytes = audio_payload
            .iter()
            .flat_map(|(bytes, len)| &bytes[*len..])
            .copied()
            .collect();

        Ok(SynthesizedAudio {
            audio_format: config.audio_format.clone(),
            audio_bytes,
            audio_metadata,
        })
    }
}

fn build_websocket_request() -> anyhow::Result<Request> {
    let uuid = uuid::Uuid::new_v4().simple().to_string();
    let mut request = format!("{}{}", constants::WSS_URL, uuid).into_client_request()?;
    let headers = request.headers_mut();
    headers.insert(header::PRAGMA, "no-cache".parse()?);
    headers.insert(header::CACHE_CONTROL, "no-cache".parse()?);
    headers.insert(header::USER_AGENT, constants::USER_AGENT.parse()?);
    headers.insert(header::ORIGIN, constants::ORIGIN.parse()?);
    Ok(request)
}

#[cfg(not(feature = "ssl-key-log"))]
fn websocket_connect() -> Result<WebSocket<MaybeTlsStream<TcpStream>>> {
    let request = build_websocket_request()?;
    let (websocket, _) = tungstenite::connect(request)?;
    Ok(websocket)
}

#[cfg(feature = "ssl-key-log")]
fn websocket_connect() -> anyhow::Result<WebSocket<MaybeTlsStream<TcpStream>>> {
    let request = build_websocket_request()?;
    let stream = try_tcp_connect(&request)?;
    let connector = build_rustls_connector();
    let (websocket, _) = tungstenite::client_tls_with_config(request, stream, None, connector)?;
    Ok(websocket)
}

#[cfg(feature = "ssl-key-log")]
fn try_tcp_connect(request: &Request) -> anyhow::Result<TcpStream> {
    use std::net::ToSocketAddrs;
    use tungstenite::error::UrlError;
    use tungstenite::stream::NoDelay;
    use tungstenite::Error;

    let uri = request.uri();
    let host = uri.host().ok_or(Error::Url(UrlError::NoHostName))?;
    let addrs = (host, 443).to_socket_addrs()?;
    for addr in addrs {
        if let Ok(mut stream) = TcpStream::connect(addr) {
            NoDelay::set_nodelay(&mut stream, true)?;
            return Ok(stream);
        }
    }
    anyhow::bail!("failed to connect to {}", uri)
}

#[cfg(feature = "ssl-key-log")]
fn build_rustls_connector() -> Option<tungstenite::Connector> {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let mut client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    client_config.key_log = std::sync::Arc::new(rustls::KeyLogFile::new());
    Some(tungstenite::Connector::Rustls(std::sync::Arc::new(
        client_config,
    )))
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
