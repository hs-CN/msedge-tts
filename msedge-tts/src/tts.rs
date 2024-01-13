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

type TungsteniteStream =
    tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>;

pub struct MSEdgeTTS(TungsteniteStream);

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
            let response = process_message(message, &mut turn_start, &mut response, &mut turn_end)?;
            if let Some(response) = response {
                match response {
                    SynthesizedResponse::AudioBytes(payload) => {
                        audio_bytes.push(payload);
                    }
                    SynthesizedResponse::AudioMetadata(metadata) => {
                        audio_metadata.extend(metadata);
                    }
                }
            }
        }

        let audio_bytes = audio_bytes
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

fn build_websocket_request() -> anyhow::Result<tungstenite::handshake::client::Request> {
    use tungstenite::client::IntoClientRequest;
    use tungstenite::http::header;

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
fn websocket_connect() -> anyhow::Result<TungsteniteStream> {
    let request = build_websocket_request()?;
    let (websocket, _) = tungstenite::connect(request)?;
    Ok(websocket)
}

#[cfg(feature = "ssl-key-log")]
fn websocket_connect() -> anyhow::Result<TungsteniteStream> {
    let request = build_websocket_request()?;
    let stream = try_connect(&request)?;
    let connector = build_rustls_connector()?;
    let (websocket, _) =
        tungstenite::client_tls_with_config(request, stream, None, Some(connector))?;
    Ok(websocket)
}

#[cfg(feature = "ssl-key-log")]
fn try_connect(
    request: &tungstenite::handshake::client::Request,
) -> anyhow::Result<std::net::TcpStream> {
    use tungstenite::error::UrlError;
    use tungstenite::Error;

    let uri = request.uri();
    let host = uri.host().ok_or(Error::Url(UrlError::NoHostName))?;
    let stream = std::net::TcpStream::connect((host, 443))?;
    stream.set_nodelay(true)?;
    Ok(stream)
}

#[cfg(feature = "ssl-key-log")]
fn build_rustls_connector() -> anyhow::Result<tungstenite::Connector> {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_parsable_certificates(rustls_native_certs::load_native_certs()?);
    let mut client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    client_config.key_log = std::sync::Arc::new(rustls::KeyLogFile::new());
    Ok(tungstenite::Connector::Rustls(std::sync::Arc::new(
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

enum SynthesizedResponse {
    AudioBytes((Vec<u8>, usize)),
    AudioMetadata(Vec<AudioMetadata>),
}

fn process_message(
    message: tungstenite::Message,
    turn_start: &mut bool,
    response: &mut bool,
    turn_end: &mut bool,
) -> anyhow::Result<Option<SynthesizedResponse>> {
    match message {
        tungstenite::Message::Text(text) => {
            if text.contains("audio.metadata") {
                if let Some(index) = text.find("\r\n\r\n") {
                    Ok(Some(SynthesizedResponse::AudioMetadata(
                        AudioMetadata::from_str(&text[index + 4..])?,
                    )))
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
                anyhow::bail!("unexpected text message: {}", text)
            }
        }
        tungstenite::Message::Binary(bytes) => {
            if *turn_start || *response {
                let header_len = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
                Ok(Some(SynthesizedResponse::AudioBytes((
                    bytes,
                    header_len + 2,
                ))))
            } else {
                Ok(None)
            }
        }
        tungstenite::Message::Close(_) => {
            *turn_end = true;
            Ok(None)
        }
        _ => anyhow::bail!("unexpected message: {}", message),
    }
}

#[cfg(not(feature = "ssl-key-log"))]
type AsyncTungsteniteStream =
    async_tungstenite::WebSocketStream<async_tungstenite::async_std::ConnectStream>;
#[cfg(feature = "ssl-key-log")]
type AsyncTungsteniteStream = async_tungstenite::WebSocketStream<
    async_tungstenite::async_tls::ClientStream<async_net::TcpStream>,
>;

pub struct MSEdgeTTSAsync(AsyncTungsteniteStream);

impl MSEdgeTTSAsync {
    pub async fn connect_async() -> anyhow::Result<Self> {
        Ok(Self(websocket_connect_asnyc().await?))
    }

    pub async fn synthesize_async(
        &mut self,
        text: &str,
        config: &SpeechConfig,
    ) -> anyhow::Result<SynthesizedAudio> {
        use futures_util::{SinkExt, StreamExt};

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
                        SynthesizedResponse::AudioBytes(payload) => {
                            audio_bytes.push(payload);
                        }
                        SynthesizedResponse::AudioMetadata(metadata) => {
                            audio_metadata.extend(metadata);
                        }
                    }
                }
            }
        }

        let audio_bytes = audio_bytes
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

#[cfg(not(feature = "ssl-key-log"))]
async fn websocket_connect_asnyc() -> anyhow::Result<AsyncTungsteniteStream> {
    let request = build_websocket_request()?;
    let (websocket, _) = async_tungstenite::async_std::connect_async(request).await?;
    Ok(websocket)
}

#[cfg(feature = "ssl-key-log")]
async fn websocket_connect_asnyc() -> anyhow::Result<AsyncTungsteniteStream> {
    use async_tungstenite::async_tls::client_async_tls_with_connector;

    let request = build_websocket_request()?;
    let stream = try_connect_async(&request).await?;
    let connector = build_async_tls_connector()?;
    let (websocket, _) = client_async_tls_with_connector(request, stream, Some(connector)).await?;
    Ok(websocket)
}

#[cfg(feature = "ssl-key-log")]
async fn try_connect_async(
    request: &tungstenite::handshake::client::Request,
) -> anyhow::Result<async_net::TcpStream> {
    use tungstenite::error::UrlError;
    use tungstenite::Error;

    let uri = request.uri();
    let host = uri.host().ok_or(Error::Url(UrlError::NoHostName))?;
    let stream = async_net::TcpStream::connect((host, 443)).await?;
    Ok(stream)
}

#[cfg(feature = "ssl-key-log")]
fn build_async_tls_connector() -> anyhow::Result<async_tls::TlsConnector> {
    let certs: Vec<_> = rustls_native_certs::load_native_certs()?
        .into_iter()
        .map(|x| x.to_vec())
        .collect();
    let mut root_store = old_rustls::RootCertStore::empty();
    root_store.add_parsable_certificates(&certs);
    let mut client_config = old_rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    client_config.key_log = std::sync::Arc::new(old_rustls::KeyLogFile::new());
    Ok(async_tls::TlsConnector::from(client_config))
}
