use mp4lib::streaming::hls;

use crate::models::*;

fn hls_extx_to_audio(t: hls::ExtXMedia) -> AudioTrack {
    AudioTrack {
        track_id: t.track_id,
        codec: t.codec,
        channels: t.channels.unwrap_or(2),
        language: t.language,
        commentary: t.commentary,
    }
}

fn hls_extx_to_subtitle(t: hls::ExtXMedia) -> SubtitleTrack {
    SubtitleTrack {
        track_id: t.track_id,
        language: t.language,
        forced: t.forced,
        sdh: t.sdh,
        commentary: t.commentary,
    }
}

fn hls_video_to_video(t: hls::Video) -> VideoTrack {
    VideoTrack {
        track_id: t.track_id,
        width: t.resolution.0,
        height: t.resolution.1,
        codec: t.codec,
    }
}

fn from_hls(hls: hls::HlsMaster) -> Video {
    Video {
        audio_tracks: hls.audio_tracks.into_iter().map(hls_extx_to_audio).collect(),
        subtitle_tracks: hls.subtitles.into_iter().map(hls_extx_to_subtitle).collect(),
        video_track: hls.video.map(hls_video_to_video),
        ..Video::default()
    }
}

pub async fn probe(video: &str) -> std::io::Result<Video> {
    use mp4lib::{io::Mp4File, mp4box::MP4};
    let hls = tokio::task::block_in_place(move || {
        let mut reader = Mp4File::open(video, false)?;
        let mp4 = MP4::read(&mut reader)?;
        let hls = hls::HlsMaster::new(&mp4, false);
        Ok::<_, std::io::Error>(hls)
    })?;
    Ok(from_hls(hls))
}
