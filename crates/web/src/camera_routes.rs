use axum::response::Response;
use axum::body::Body;
use axum::http::header;
use std::convert::Infallible;
use std::sync::Arc;

use crate::state::AppState;

pub async fn mjpeg_stream(
    axum::Extension(state): axum::Extension<Arc<AppState>>,
) -> Response {
    let config = state.config.read().await.camera.clone();

    let (stream, _shutdown_rx) = pi_kiosk_camera::MjpegStream::spawn(config);

    let body = Body::from_stream(stream);

    Response::builder()
        .header(header::CONTENT_TYPE, "multipart/x-mixed-replace; boundary=frame")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .header(header::PRAGMA, "no-cache")
        .body(body)
        .unwrap()
}

pub async fn clip_file(
    axum::extract::Path(filename): axum::extract::Path<String>,
) -> Result<Response, Infallible> {
    let config = pi_kiosk_core::AppConfig::default();
    let mgr = pi_kiosk_camera::ClipManager::new(
        &config.storage.clip_dir,
        config.storage.max_retention_days,
        config.storage.max_disk_usage_pct,
    );

    if let Some(path) = mgr.clip_path(&filename) {
        match tokio::fs::read(&path).await {
            Ok(data) => {
                let content_type = if filename.ends_with(".jpg") {
                    "image/jpeg"
                } else if filename.ends_with(".h264") {
                    "video/h264"
                } else if filename.ends_with(".mp4") {
                    "video/mp4"
                } else {
                    "application/octet-stream"
                };
                Ok(
                    Response::builder()
                        .header(header::CONTENT_TYPE, content_type)
                        .header(header::CACHE_CONTROL, "max-age=3600")
                        .body(Body::from(data))
                        .unwrap(),
                )
            }
            Err(_) => Ok(
                Response::builder()
                    .status(404)
                    .body(Body::from("not found"))
                    .unwrap(),
            ),
        }
    } else {
        Ok(
            Response::builder()
                .status(404)
                .body(Body::from("not found"))
                .unwrap(),
        )
    }
}
