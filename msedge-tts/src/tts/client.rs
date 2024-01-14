use super::{
    build_config_message, build_ssml_message, process_message, websocket_connect,
    websocket_connect_asnyc, AudioMetadata, ProcessedMessage, SpeechConfig, WebSocketStream,
    WebSocketStreamAsync,
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

#[derive(Debug)]
pub struct SynthesizedAudio {
    pub audio_format: String,
    pub audio_bytes: Vec<u8>,
    pub audio_metadata: Vec<AudioMetadata>,
}
