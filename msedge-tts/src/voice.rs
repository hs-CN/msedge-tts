use anyhow::Result;
use isahc::{ReadResponseExt, Request, RequestExt};
use serde::Deserialize;

use crate::constants;

#[derive(Debug, Deserialize)]
pub struct VoiceTag {
    #[serde(rename = "ContentCategories")]
    pub content_categories: Option<Vec<String>>,
    #[serde(rename = "VoicePersonalities")]
    pub voice_personalities: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
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

pub fn get_voices_list() -> Result<Vec<Voice>> {
    let body = Request::get(constants::VOICE_LIST_URL)
        .header("Sec-CH-UA", constants::SEC_CH_UA)
        .header("Sec-CH-UA-Mobile", constants::SEC_CH_UA_MOBILE)
        .header("User-Agent", constants::USER_AGENT)
        .header("Sec-CH-UA-Platform", constants::SEC_CH_UA_PLATFORM)
        .header("Sec-Fetch-Site", constants::SEC_FETCH_SITE)
        .header("Sec-Fetch-Mode", constants::SEC_FETCH_MODE)
        .header("Sec-Fetch-Dest", constants::SEC_FETCH_DEST)
        .body(())?
        .send()?
        .json()?;
    Ok(body)
}
