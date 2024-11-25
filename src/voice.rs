//! Voice Type and Get function
//!
//! Use [get_voices_list] function to get all available voices.  
//! Use [get_voices_list_async] function to get all available voices asynchronously.  
//! Use [get_voices_list_proxy] function to get all available voices with proxy.  
//! Use [get_voices_list_proxy_async] function to get all available voices with proxy asynchronously.

use crate::{constants, error::Result};
use isahc::{config::Configurable, AsyncReadResponseExt, ReadResponseExt, RequestExt};

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

impl From<String> for Voice {
    fn from(voice_name: String) -> Self {
        Self {
            name: voice_name,
            short_name: None,
            gender: None,
            locale: None,
            suggested_codec: None,
            friendly_name: None,
            status: None,
            voice_tag: None,
        }
    }
}

impl From<&str> for Voice {
    fn from(voice_name: &str) -> Self {
        voice_name.to_string().into()
    }
}

/// Get all available voices
pub fn get_voices_list() -> Result<Vec<Voice>> {
    Ok(build_request(None, None, None)
        .map_err(isahc::Error::from)?
        .send()?
        .json()?)
}

/// Get all available voices with proxy.
///
/// **docs copy from isahc**  
/// Set a proxy to use for requests.
/// The proxy protocol is specified by the URI scheme.
///
/// `http`: Proxy. Default when no scheme is specified.  
/// `https`: HTTPS Proxy. (Added in 7.52.0 for OpenSSL, GnuTLS and NSS)  
/// `socks4`: SOCKS4 Proxy.  
/// `socks4a`: SOCKS4a Proxy. Proxy resolves URL hostname.  
/// `socks5`: SOCKS5 Proxy.  
/// `socks5h`: SOCKS5 Proxy. Proxy resolves URL hostname.  
pub fn get_voices_list_proxy(
    proxy: isahc::http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<Vec<Voice>> {
    Ok(build_request(Some(proxy), username, password)
        .map_err(isahc::Error::from)?
        .send()?
        .json()?)
}

/// Get all available voices asynchronously
pub async fn get_voices_list_async() -> Result<Vec<Voice>> {
    Ok(build_request(None, None, None)
        .map_err(isahc::Error::from)?
        .send_async()
        .await?
        .json()
        .await?)
}

/// Get all available voices asynchronously with proxy.
///
/// **docs copy from isahc**  
/// Set a proxy to use for requests.
/// The proxy protocol is specified by the URI scheme.
///
/// `http`: Proxy. Default when no scheme is specified.  
/// `https`: HTTPS Proxy. (Added in 7.52.0 for OpenSSL, GnuTLS and NSS)  
/// `socks4`: SOCKS4 Proxy.  
/// `socks4a`: SOCKS4a Proxy. Proxy resolves URL hostname.  
/// `socks5`: SOCKS5 Proxy.  
/// `socks5h`: SOCKS5 Proxy. Proxy resolves URL hostname.  
pub async fn get_voices_list_proxy_async(
    proxy: isahc::http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<Vec<Voice>> {
    Ok(build_request(Some(proxy), username, password)
        .map_err(isahc::Error::from)?
        .send_async()
        .await?
        .json()
        .await?)
}

fn build_request(
    proxy: Option<isahc::http::Uri>,
    username: Option<&str>,
    password: Option<&str>,
) -> std::result::Result<isahc::Request<()>, isahc::http::Error> {
    let mut builder = isahc::Request::get(constants::VOICE_LIST_URL)
        .header("Sec-CH-UA", constants::SEC_CH_UA)
        .header("Sec-CH-UA-Mobile", constants::SEC_CH_UA_MOBILE)
        .header("User-Agent", constants::USER_AGENT)
        .header("Sec-CH-UA-Platform", constants::SEC_CH_UA_PLATFORM)
        .header("Sec-Fetch-Site", constants::SEC_FETCH_SITE)
        .header("Sec-Fetch-Mode", constants::SEC_FETCH_MODE)
        .header("Sec-Fetch-Dest", constants::SEC_FETCH_DEST);

    if proxy.is_some() {
        builder = builder.proxy(proxy);
        if username.is_some() && password.is_some() {
            builder = builder.proxy_authentication(isahc::auth::Authentication::basic());
            builder = builder.proxy_credentials(isahc::auth::Credentials::new(
                username.unwrap(),
                password.unwrap(),
            ));
        }
    }

    builder.body(())
}
