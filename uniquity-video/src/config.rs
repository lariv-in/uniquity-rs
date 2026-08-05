use serde::{Deserialize, Serialize};

use lariv_rs::config::ConfigSection;

pub struct VideoConfigTag;

impl ConfigSection for VideoConfigTag {
    const KEY: Option<&'static str> = Some("uniquity_video");
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct VideoConfig {
    #[serde(default, rename = "youtubeApiKey")]
    pub youtube_api_key: String,
}

impl VideoConfig {
    pub fn youtube_api_key(&self) -> &str {
        self.youtube_api_key.trim()
    }
}
