pub mod voice;

use anyhow::{bail, Result};
use chrono::Local;
use serde_json::from_str;
use std::{
    net::{TcpStream, ToSocketAddrs},
    sync::Arc,
};
use tungstenite::{
    client::{uri_mode, IntoClientRequest},
    client_tls_with_config, error,
    handshake::client::Request,
    http::header,
    stream::{MaybeTlsStream, NoDelay},
    Connector, Error, WebSocket,
};

static USER_AGENT:&str="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";

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

impl From<&voice::Voice> for SpeechConfig {
    fn from(voice: &voice::Voice) -> Self {
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

impl From<voice::Voice> for SpeechConfig {
    fn from(voice: voice::Voice) -> Self {
        let audio_output_format = if let Some(output_format) = voice.suggested_codec {
            output_format
        } else {
            "audio-24khz-48kbitrate-mono-mp3".to_string()
        };

        Self {
            voice_name: voice.name,
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
    fn from_str(text: &str) -> Result<Vec<Self>> {
        let value: serde_json::Value = from_str(text)?;
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
            bail!("unexpected json text: {:#?}", value)
        }
    }
}

#[derive(Debug)]
pub struct SynthesizedAudio {
    pub audio_bytes: Vec<u8>,
    pub audio_metadata: Vec<AudioMetadata>,
    pub audio_format: String,
}

pub struct MSEdgeTTS(WebSocket<MaybeTlsStream<TcpStream>>);

impl MSEdgeTTS {
    pub fn connect() -> Result<Self> {
        let request = Self::build_request()?;
        let stream = Self::try_connect(&request)?;
        let connector = Self::build_connector();
        let (websocket, _) = client_tls_with_config(request, stream, None, connector)?;
        Ok(Self(websocket))
    }

    fn build_request() -> Result<Request> {
        static WSS_URL:&str="wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1?TrustedClientToken=6A5AA1D4EAFF4E9FB37E23D68491D6F4&ConnectionId=";
        static ORIGIN: &str = "chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold";
        let uuid = uuid::Uuid::new_v4().simple().to_string();

        let mut request = format!("{}{}", WSS_URL, uuid).into_client_request()?;
        let headers = request.headers_mut();
        headers.insert(header::PRAGMA, "no-cache".parse()?);
        headers.insert(header::CACHE_CONTROL, "no-cache".parse()?);
        headers.insert(header::USER_AGENT, USER_AGENT.parse()?);
        headers.insert(header::ORIGIN, ORIGIN.parse()?);

        Ok(request)
    }

    fn try_connect(request: &Request) -> Result<TcpStream> {
        let uri = request.uri();
        let mode = uri_mode(uri)?;
        let host = request
            .uri()
            .host()
            .ok_or(Error::Url(error::UrlError::NoHostName))?;
        let host = if host.starts_with('[') {
            &host[1..host.len() - 1]
        } else {
            host
        };
        let port = uri.port_u16().unwrap_or(match mode {
            tungstenite::stream::Mode::Plain => 80,
            tungstenite::stream::Mode::Tls => 443,
        });
        let addrs = (host, port).to_socket_addrs()?;
        for addr in addrs {
            if let Ok(mut stream) = TcpStream::connect(addr) {
                NoDelay::set_nodelay(&mut stream, true)?;
                return Ok(stream);
            }
        }
        bail!("failed to connect to {}", uri)
    }

    fn build_connector() -> Option<Connector> {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let mut client_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        // enabel SSLKEYLOGFILE
        client_config.key_log = Arc::new(rustls::KeyLogFile::new());
        Some(Connector::Rustls(Arc::new(client_config)))
    }

    pub fn synthesize(&mut self, text: &str, config: &SpeechConfig) -> Result<SynthesizedAudio> {
        static SPEECH_CONFIG_HEAD: &str = r#"{"context":{"synthesis":{"audio":{"metadataoptions":{"sentenceBoundaryEnabled":"false","wordBoundaryEnabled":"true"},"outputFormat":""#;
        static SPEECH_CONFIG_TAIL: &str = r#""}}}}"#;
        let speech_config = format!(
            "{}{}{}",
            SPEECH_CONFIG_HEAD, config.audio_format, SPEECH_CONFIG_TAIL
        );

        let speech_config_message=format!(
            "X-Timestamp:{}\r\nContent-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n{}",
            Local::now().to_rfc2822(),
            speech_config
        );

        self.0.write(speech_config_message.into())?;

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
            Local::now().to_rfc2822(),
            ssml,
        );

        self.0.write(ssml_message.into())?;
        self.0.flush()?;

        let mut turn_start = false;
        let mut response = false;
        let mut audio_payload = Vec::new();
        let mut audio_metadata = Vec::new();
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
                        bail!("unexpected text message: {}", text)
                    }
                }
                tungstenite::Message::Binary(bytes) => {
                    if turn_start || response {
                        let header_len = u16::from_be_bytes(bytes[0..2].try_into()?) as usize;
                        audio_payload.push((bytes, header_len + 2));
                    }
                }
                tungstenite::Message::Close(_) => break,
                _ => bail!("unexpected message: {}", message),
            }
        }
        let audio_bytes: Vec<u8> = audio_payload
            .iter()
            .flat_map(|(bytes, index)| &bytes[*index..])
            .copied()
            .collect();

        Ok(SynthesizedAudio {
            audio_bytes,
            audio_metadata,
            audio_format: config.audio_format.clone(),
        })
    }
}
