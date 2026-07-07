mod app;
mod auth;
mod camera_routes;
mod pages;
mod priv_client;
mod server_fns;
mod sse;
mod state;

use axum::Router;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use pi_kiosk_core::AppConfig;
use pi_kiosk_db::Database;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pi_kiosk_web=info,tower_http=info".into()),
        )
        .init();

    let config = AppConfig::default();
    let db = Database::open(&config.db_path)?;
    let live = state::create_live_state();
    let state = Arc::new(AppState::new(config.clone(), db, live));

    // Spawn background workers
    crate::state::spawn_workers(state.clone()).await;

    let leptos_opts = get_configuration(None)
        .await?
        .leptos_options;
    let addr = config.listen_addr.clone();

    let routes = generate_route_list(|| view! { <app::App/> });

    let app = Router::new()
        .leptos_routes(&leptos_opts, routes, || view! { <app::App/> })
        .route("/api/events", axum::routing::get(sse::events_sse))
        .route("/api/stream", axum::routing::get(camera_routes::mjpeg_stream))
        .route("/api/clips/{filename}", axum::routing::get(camera_routes::clip_file))
        .with_state(leptos_opts)
        .layer(TraceLayer::new_for_http())
        .layer(axum::Extension(state.clone()))
        .layer(axum::middleware::from_fn(auth::require_auth));

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let tls_config = state.config.read().await.tls.clone();

    if tls_config.enabled && !tls_config.cert_path.is_empty() && !tls_config.key_path.is_empty() {
        tracing::info!("pi-kiosk-web listening on https://{}", addr);

        let cert_pem = std::fs::read(&tls_config.cert_path)
            .map_err(|e| anyhow::anyhow!("failed to read cert: {e}"))?;
        let key_pem = std::fs::read(&tls_config.key_path)
            .map_err(|e| anyhow::anyhow!("failed to read key: {e}"))?;

        let certs: Vec<rustls::pki_types::CertificateDer> =
            rustls_pemfile::certs(&mut cert_pem.as_slice())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| anyhow::anyhow!("failed to parse cert: {e}"))?;

        let key = rustls_pemfile::pkcs8_private_keys(&mut key_pem.as_slice())
            .next()
            .ok_or_else(|| anyhow::anyhow!("no PKCS8 key found in key file"))?
            .map_err(|e| anyhow::anyhow!("failed to parse key: {e}"))?;

        let rustls_config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, rustls::pki_types::PrivateKeyDer::Pkcs8(key))
            .map_err(|e| anyhow::anyhow!("failed to build TLS config: {e}"))?;

        let acceptor = tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(rustls_config));

        loop {
            let (stream, _peer) = listener.accept().await?;
            let acceptor = acceptor.clone();
            let app = app.clone();
            tokio::spawn(async move {
                if let Ok(tls_stream) = acceptor.accept(stream).await {
                    let io = hyper_util::rt::TokioIo::new(tls_stream);
                    let svc = hyper_util::service::TowerToHyperService::new(app.into_service());
                    let _ = hyper::server::conn::http1::Builder::new()
                        .serve_connection(io, svc)
                        .await;
                }
            });
        }
    } else {
        tracing::info!("pi-kiosk-web listening on http://{}", addr);
        axum::serve(listener, app).await?;
    }

    Ok(())
}
