use std::{path::PathBuf, process::Stdio};

use axum::{
    body::Body,
    extract::{Multipart, Query},
    http::{header, HeaderMap, HeaderValue, Response, StatusCode},
};
use serde::Deserialize;
use tokio::{fs, process::Command};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};

#[derive(Debug, Deserialize)]
pub struct ConvertQuery {
    format: Option<String>,
}

pub async fn video_to_audio(
    Query(query): Query<ConvertQuery>,
    mut multipart: Multipart,
) -> ApiResult<Response<Body>> {
    let format = AudioFormat::from_query(query.format.as_deref())?;
    let mut file_bytes = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::BadRequest("invalid multipart body".into()))?
    {
        if field.name() == Some("file") {
            file_bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|_| ApiError::BadRequest("read upload failed".into()))?,
            );
            break;
        }
    }

    let file_bytes = file_bytes.ok_or_else(|| ApiError::BadRequest("file is required".into()))?;
    if file_bytes.is_empty() {
        return Err(ApiError::BadRequest("file is empty".into()));
    }

    let job_id = Uuid::new_v4().to_string();
    let input_path = std::env::temp_dir().join(format!("mobile-video-{job_id}.upload"));
    let output_path =
        std::env::temp_dir().join(format!("mobile-audio-{job_id}.{}", format.extension()));

    fs::write(&input_path, file_bytes)
        .await
        .map_err(|_| ApiError::Internal)?;
    let result = run_ffmpeg(&input_path, &output_path, format).await;
    let _ = fs::remove_file(&input_path).await;

    if let Err(err) = result {
        let _ = fs::remove_file(&output_path).await;
        return Err(err);
    }

    let output = fs::read(&output_path)
        .await
        .map_err(|_| ApiError::Internal)?;
    let _ = fs::remove_file(&output_path).await;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(format.content_type()),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename=\"converted-audio.{}\"",
            format.extension()
        ))
        .map_err(|_| ApiError::Internal)?,
    );

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(output))
        .map(|mut resp| {
            *resp.headers_mut() = headers;
            resp
        })
        .map_err(|_| ApiError::Internal)
}

async fn run_ffmpeg(
    input_path: &PathBuf,
    output_path: &PathBuf,
    format: AudioFormat,
) -> ApiResult<()> {
    let mut command = Command::new("ffmpeg");
    command.arg("-y").arg("-i").arg(input_path).arg("-vn");
    match format {
        AudioFormat::M4a => {
            command.args(["-c:a", "aac", "-b:a", "192k"]);
        }
        AudioFormat::Mp3 => {
            command.args(["-c:a", "libmp3lame", "-q:a", "2"]);
        }
        AudioFormat::Wav => {
            command.args(["-c:a", "pcm_s16le"]);
        }
    }

    let output = command
        .arg(output_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|_| ApiError::Internal)?;

    if output.status.success() {
        Ok(())
    } else {
        let message = String::from_utf8_lossy(&output.stderr)
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("ffmpeg conversion failed")
            .to_string();
        Err(ApiError::BadRequest(message))
    }
}

#[derive(Debug, Clone, Copy)]
enum AudioFormat {
    M4a,
    Mp3,
    Wav,
}

impl AudioFormat {
    fn from_query(value: Option<&str>) -> ApiResult<Self> {
        match value.unwrap_or("m4a").to_ascii_lowercase().as_str() {
            "m4a" => Ok(Self::M4a),
            "mp3" => Ok(Self::Mp3),
            "wav" => Ok(Self::Wav),
            _ => Err(ApiError::BadRequest("unsupported audio format".into())),
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Self::M4a => "m4a",
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
        }
    }

    fn content_type(self) -> &'static str {
        match self {
            Self::M4a => "audio/mp4",
            Self::Mp3 => "audio/mpeg",
            Self::Wav => "audio/wav",
        }
    }
}
