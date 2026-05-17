//! Voice Type and Get function
//!
//! Use [get_voices_list] function to get all available voices.  
//! Use [get_voices_list_async] function to get all available voices asynchronously.  
//! Use [get_voices_list_proxy] function to get all available voices with proxy.  
//! Use [get_voices_list_proxy_async] function to get all available voices with proxy asynchronously.

use crate::{constants, error::Result};
use isahc::{AsyncReadResponseExt, RequestExt, config::Configurable};

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
