pub const VOICE_LIST_URL: &str = "https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list?trustedclienttoken=6A5AA1D4EAFF4E9FB37E23D68491D6F4";
// Updated USER_AGENT to reflect the current Microsoft Edge stable (155.0.4283.45).
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/155.0.0.0 Safari/537.36 Edg/155.0.0.0";
pub const WSS_URL: &str = "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1?TrustedClientToken=6A5AA1D4EAFF4E9FB37E23D68491D6F4&ConnectionId=";
// The `2-` prefix selects the v2 Sec-MS-GEC token format and must match it; the
// trailing version number is ignored by the service.
// Solution source: https://github.com/rany2/edge-tts/pull/492
pub const SEC_MS_GEC_VERSION: &str = "2-155.0.4283.45";
pub const ORIGIN: &str = "chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold";
