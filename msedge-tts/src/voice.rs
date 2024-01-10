use anyhow::Result;
use serde::Deserialize;

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
    static VOICE_LIST_URL:&str="https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list?trustedclienttoken=6A5AA1D4EAFF4E9FB37E23D68491D6F4";
    static SEC_CH_UA: &str = r#""Not_A Brand";v="8", "Chromium";v="120", "Microsoft Edge";v="120""#;
    static SEC_CH_UA_MOBILE: &str = "?0";
    static SEC_CH_UA_PLATFORM: &str = r#""Windows""#;
    static SEC_FETCH_SITE: &str = "none";
    static SEC_FETCH_MODE: &str = "cors";
    static SEC_FETCH_DEST: &str = "empty";
    let body: Vec<Voice> = ureq::get(VOICE_LIST_URL)
        .set("Sec-CH-UA", SEC_CH_UA)
        .set("Sec-CH-UA-Mobile", SEC_CH_UA_MOBILE)
        .set("User-Agent", super::USER_AGENT)
        .set("Sec-CH-UA-Platform", SEC_CH_UA_PLATFORM)
        .set("Sec-Fetch-Site", SEC_FETCH_SITE)
        .set("Sec-Fetch-Mode", SEC_FETCH_MODE)
        .set("Sec-Fetch-Dest", SEC_FETCH_DEST)
        .call()?
        .into_json()?;
    Ok(body)
}
