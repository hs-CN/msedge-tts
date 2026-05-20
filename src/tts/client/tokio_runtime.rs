use futures_util::{StreamExt, io::AsyncRead, io::AsyncWrite};

use crate::{
    error::Result,
    tts::{
        Payload, SpeechConfig, build_config_message, build_ssml_message, client::SynthesizedAudio,
        websocket_connect_tokio_async,
    },
};

pub struct MSEdgeTTSClientAsync<T>(pub(crate) async_tungstenite::WebSocketStream<T>);

impl<T: AsyncRead + AsyncWrite + Unpin> MSEdgeTTSClientAsync<T> {
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
                let payload =
                    Payload::process(message, &mut turn_start, &mut response, &mut turn_end)?;
                if let Some(payload) = payload {
                    match payload {
                        Payload::AudioBytes(payload) => {
                            audio_bytes.push(payload);
                        }
                        Payload::AudioMetadata(metadata) => {
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

pub async fn connect_async() -> Result<MSEdgeTTSClientAsync<async_tungstenite::tokio::ConnectStream>>
{
    Ok(MSEdgeTTSClientAsync(websocket_connect_tokio_async().await?))
}

#[cfg(feature = "proxy")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "tokio-runtime", feature = "proxy"))))]
pub use crate::tts::proxy::tokio_runtime::connect_proxy_async;
