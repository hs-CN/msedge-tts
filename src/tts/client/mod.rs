//! TTS Client module

use crate::tts::AudioMetadata;

/// Synthesized Audio and Metadata
#[derive(Debug)]
pub struct SynthesizedAudio {
    pub audio_format: String,
    pub audio_bytes: Vec<u8>,
    pub audio_metadata: Vec<AudioMetadata>,
}

#[cfg(feature = "blocking")]
mod blocking;
#[cfg(feature = "blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
pub use blocking::*;

#[cfg(feature = "smol-runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "smol-runtime")))]
pub mod soml_runtime;
