use super::{
    build_config_message, build_ssml_message, websocket_connect, websocket_connect_asnyc,
    AudioMetadata, SpeechConfig, WebSocketStream, WebSocketStreamAsync,
};

pub struct MSEdgeTTSClient(WebSocketStream);

impl MSEdgeTTSClient {
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

pub struct MSEdgeTTSClientAsync(WebSocketStreamAsync);

impl MSEdgeTTSClientAsync {
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

pub struct SynthesizedAudio {
    pub audio_format: String,
    pub audio_bytes: Vec<u8>,
    pub audio_metadata: Vec<AudioMetadata>,
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
