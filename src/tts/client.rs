//! Synthesis Client

use super::{
    build_config_message, build_ssml_message, process_message, websocket_connect,
    websocket_connect_asnyc, AudioMetadata, ProcessedMessage, SpeechConfig, WebSocketStream,
    WebSocketStreamAsync,
};
use crate::error::Result;

/// Sync Client
pub struct MSEdgeTTSClient(WebSocketStream);

impl MSEdgeTTSClient {
    /// Create a new sync Client
    pub fn connect() -> Result<Self> {
        Ok(Self(websocket_connect()?))
    }

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
pub struct MSEdgeTTSClientAsync(WebSocketStreamAsync);

impl MSEdgeTTSClientAsync {
    /// Create a new async Client
    pub async fn connect_async() -> Result<Self> {
        Ok(Self(websocket_connect_asnyc().await?))
    }

    /// Synthesize text to speech with a [SpeechConfig] asynchronously
    pub async fn synthesize_async(
        &mut self,
        text: &str,
        config: &SpeechConfig,
    ) -> Result<SynthesizedAudio> {
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

/// Synthesized Audio and Metadata
#[derive(Debug)]
pub struct SynthesizedAudio {
    pub audio_format: String,
    pub audio_bytes: Vec<u8>,
    pub audio_metadata: Vec<AudioMetadata>,
}
