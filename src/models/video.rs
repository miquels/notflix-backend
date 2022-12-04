use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::sqlx::impl_sqlx_traits_for;
use super::is_default;

#[derive(Object, Deserialize, Serialize, Clone, Default, Debug, PartialEq)]
pub struct AudioTrack {
    pub track_id: u32,
    pub codec: String,
    pub channels: u16,
    #[oai(skip_serializing_if = "is_default")]
    pub language: Option<String>,
    #[oai(skip_serializing_if = "is_default")]
    pub commentary: bool,
}

#[derive(Object, Deserialize, Serialize, Clone, Default, Debug, PartialEq)]
pub struct SubtitleTrack {
    pub track_id: u32,
    #[oai(skip_serializing_if = "is_default")]
    pub language: Option<String>,
    #[oai(skip_serializing_if = "is_default")]
    pub forced: bool,
    #[oai(skip_serializing_if = "is_default")]
    pub sdh: bool,
    #[oai(skip_serializing_if = "is_default")]
    pub commentary: bool,
}

#[derive(Object, Deserialize, Serialize, Clone, Default, Debug, PartialEq)]
pub struct VideoTrack {
    pub track_id: u32,
    pub width:  u16,
    pub height: u16,
    pub codec: String,
}

#[derive(Object, Deserialize, Serialize, Clone, Default, Debug, PartialEq)]
pub struct Video {
    #[oai(skip_serializing_if = "is_default")]
    pub audio_tracks: Vec<AudioTrack>,
    #[oai(skip_serializing_if = "is_default")]
    pub subtitle_tracks: Vec<SubtitleTrack>,
    #[oai(skip_serializing_if = "is_default")]
    pub video_track: Option<VideoTrack>,
    #[oai(skip_serializing_if = "is_default")]
    pub path: String,
}
impl_sqlx_traits_for!(Video);
