//! Voice Type and Get function

#[cfg(feature = "smol-runtime")]
mod smol_runtime;
#[cfg(feature = "smol-runtime")]
pub use smol_runtime::*;

use crate::{constants, error::Result};

/// Voice category tags and personalities tags
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct VoiceTag {
    #[serde(rename = "ContentCategories")]
    pub content_categories: Option<Vec<String>>,
    #[serde(rename = "VoicePersonalities")]
    pub voice_personalities: Option<Vec<String>>,
}

/// Voice get from MS Edge Read aloud API.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Voice {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ShortName")]
    pub short_name: Option<String>,
    #[serde(rename = "Gender")]
    pub gender: Option<String>,
    #[serde(rename = "Locale")]
    pub locale: Option<String>,
    #[serde(rename = "SuggestedCodec")]
    pub suggested_codec: Option<String>,
    #[serde(rename = "FriendlyName")]
    pub friendly_name: Option<String>,
    #[serde(rename = "Status")]
    pub status: Option<String>,
    #[serde(rename = "VoiceTag")]
    pub voice_tag: Option<VoiceTag>,
}

/// Get all available voices
#[cfg(feature = "default")]
pub fn get_voices_list() -> Result<Vec<Voice>> {
    Ok(ureq::get(constants::VOICE_LIST_URL)
        .header("User-Agent", constants::USER_AGENT)
        .call()?
        .body_mut()
        .read_json()?)
}

/// Get all available voices with proxy.
///
/// **doc copy from ureq**
///
/// Create a proxy from a uri.
///
/// # Arguments:
///
/// * `proxy` - a str of format `<protocol>://<user>:<password>@<host>:port` . All parts
///   except host are optional.
///
/// ###  Protocols
///
/// * `http`: HTTP CONNECT proxy
/// * `https`: HTTPS CONNECT proxy (requires a TLS provider)
/// * `socks4`: SOCKS4 (requires **socks-proxy** feature)
/// * `socks4a`: SOCKS4A (requires **socks-proxy** feature)
/// * `socks5` and `socks`: SOCKS5 (requires **socks-proxy** feature)
///
/// # Examples proxy formats
///
/// * `http://127.0.0.1:8080`
/// * `socks5://john:smith@socks.google.com`
/// * `john:smith@socks.google.com:8000`
/// * `localhost`
#[cfg(feature = "default")]
pub fn get_voices_list_proxy(proxy: &str) -> Result<Vec<Voice>> {
    let proxy = ureq::Proxy::new(proxy)?;
    let config = ureq::config::Config::builder().proxy(Some(proxy)).build();
    Ok(config
        .new_agent()
        .get(constants::VOICE_LIST_URL)
        .header("User-Agent", constants::USER_AGENT)
        .call()?
        .body_mut()
        .read_json()?)
}
