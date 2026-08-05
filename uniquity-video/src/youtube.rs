//! YouTube video ID parsing and Data API v3 metadata fetch.

use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum YouTubeError {
    #[error("invalid YouTube video ID: expected 11 characters [a-zA-Z0-9_-] or a YouTube URL")]
    InvalidId,
    #[error("could not parse a YouTube video id from the URL")]
    ParseFailed,
    #[error("youtube api key not configured")]
    MissingApiKey,
    #[error("video not found or not visible with this API key")]
    NotFound,
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("api: {0}")]
    Api(String),
}

static YT_VIDEO_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_-]{11}$").unwrap());

static YT_URL_EXTRACTORS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"youtu\.be/([a-zA-Z0-9_-]{11})").unwrap(),
        Regex::new(r"youtube\.com/embed/([a-zA-Z0-9_-]{11})").unwrap(),
        Regex::new(r"youtube\.com/shorts/([a-zA-Z0-9_-]{11})").unwrap(),
        Regex::new(r"youtube\.com/live/([a-zA-Z0-9_-]{11})").unwrap(),
        Regex::new(r"[?&]v=([a-zA-Z0-9_-]{11})").unwrap(),
    ]
});

fn is_youtube_urlish(s: &str) -> bool {
    let s = s.to_lowercase();
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.contains("youtube.com")
        || s.contains("youtu.be")
}

fn parse_youtube_url(s: &str) -> Result<String, YouTubeError> {
    let trimmed = s.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Ok(trimmed.to_string());
    }
    Ok(format!("https://{}", trimmed.trim_start_matches('/')))
}

/// Normalize a bare 11-character id or YouTube URL to a video id.
pub fn clean_youtube_video_id(raw: &str) -> Result<String, YouTubeError> {
    let s = raw.trim();
    if s.is_empty() {
        return Err(YouTubeError::InvalidId);
    }
    if YT_VIDEO_ID_RE.is_match(s) {
        return Ok(s.to_string());
    }
    if !is_youtube_urlish(s) {
        return Err(YouTubeError::InvalidId);
    }
    let canonical = parse_youtube_url(s)?;
    for re in YT_URL_EXTRACTORS.iter() {
        if let Some(caps) = re.captures(&canonical) {
            if let Some(id) = caps.get(1) {
                if YT_VIDEO_ID_RE.is_match(id.as_str()) {
                    return Ok(id.as_str().to_string());
                }
            }
        }
    }
    for re in YT_URL_EXTRACTORS.iter() {
        if let Some(caps) = re.captures(s) {
            if let Some(id) = caps.get(1) {
                if YT_VIDEO_ID_RE.is_match(id.as_str()) {
                    return Ok(id.as_str().to_string());
                }
            }
        }
    }
    Err(YouTubeError::ParseFailed)
}

pub fn youtube_watch_url(video_id: &str) -> Option<String> {
    let s = video_id.trim();
    if s.is_empty() || !YT_VIDEO_ID_RE.is_match(s) {
        return None;
    }
    Some(format!("https://www.youtube.com/watch?v={s}"))
}

pub fn youtube_studio_url(video_id: &str) -> Option<String> {
    let s = video_id.trim();
    if s.is_empty() || !YT_VIDEO_ID_RE.is_match(s) {
        return None;
    }
    Some(format!("https://studio.youtube.com/video/{s}/edit"))
}

#[derive(Debug, Clone, Default)]
pub struct YouTubeSnippetMeta {
    pub title: String,
    pub published_at: String,
    pub published_at_display: String,
    pub upload_status: String,
    pub view_count: String,
    pub like_count: String,
    pub comment_count: String,
}

#[derive(Debug, Deserialize)]
struct YouTubeListResponse {
    items: Vec<YouTubeVideoItem>,
}

#[derive(Debug, Deserialize)]
struct YouTubeVideoItem {
    snippet: Option<YouTubeSnippet>,
    status: Option<YouTubeStatus>,
    statistics: Option<YouTubeStatistics>,
}

#[derive(Debug, Deserialize)]
struct YouTubeSnippet {
    title: Option<String>,
    published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YouTubeStatus {
    upload_status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YouTubeStatistics {
    view_count: Option<String>,
    like_count: Option<String>,
    comment_count: Option<String>,
}

pub fn format_upload_status_label(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() {
        return String::new();
    }
    s.replace('_', " ")
}

pub fn format_published_at_display(published_rfc3339: &str) -> String {
    let s = published_rfc3339.trim();
    if s.is_empty() {
        return String::new();
    }
    if let Ok(t) = DateTime::parse_from_rfc3339(s) {
        return t.with_timezone(&Utc).format("%Y-%m-%d %H:%M UTC").to_string();
    }
    s.to_string()
}

pub async fn fetch_youtube_snippet_meta(
    client: &reqwest::Client,
    api_key: &str,
    video_id: &str,
) -> Result<YouTubeSnippetMeta, YouTubeError> {
    let api_key = api_key.trim();
    let video_id = video_id.trim();
    if api_key.is_empty() {
        return Err(YouTubeError::MissingApiKey);
    }
    if !YT_VIDEO_ID_RE.is_match(video_id) {
        return Err(YouTubeError::InvalidId);
    }

    let url = format!(
        "https://www.googleapis.com/youtube/v3/videos?part=snippet,status,statistics&id={video_id}&key={api_key}"
    );
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(YouTubeError::Api(format!("status {}", resp.status())));
    }
    let body: YouTubeListResponse = resp.json().await?;
    let Some(v) = body.items.into_iter().next() else {
        return Err(YouTubeError::NotFound);
    };

    let mut out = YouTubeSnippetMeta::default();
    if let Some(sn) = v.snippet {
        out.title = sn.title.unwrap_or_default().trim().to_string();
        let raw = sn.published_at.unwrap_or_default();
        out.published_at = raw.trim().to_string();
        out.published_at_display = format_published_at_display(&out.published_at);
    }
    if let Some(st) = v.status {
        out.upload_status = format_upload_status_label(&st.upload_status.unwrap_or_default());
    }
    if let Some(stats) = v.statistics {
        out.view_count = stats.view_count.unwrap_or_else(|| "0".into());
        out.like_count = stats.like_count.unwrap_or_else(|| "0".into());
        out.comment_count = stats.comment_count.unwrap_or_else(|| "0".into());
    }
    Ok(out)
}

pub fn dash_if_empty(s: &str) -> &str {
    if s.trim().is_empty() {
        "—"
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bare_id() {
        assert_eq!(
            clean_youtube_video_id("dQw4w9WgXcQ").unwrap(),
            "dQw4w9WgXcQ"
        );
    }

    #[test]
    fn parses_watch_url() {
        assert_eq!(
            clean_youtube_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap(),
            "dQw4w9WgXcQ"
        );
    }
}
