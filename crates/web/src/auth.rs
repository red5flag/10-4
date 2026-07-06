use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use std::sync::Arc;

use crate::state::AppState;

pub const SESSION_COOKIE_NAME: &str = "pi-kiosk-session";
pub const LOGIN_PATH: &str = "/login";

pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path();

    // Allow public paths
    if is_public_path(path) {
        return next.run(req).await;
    }

    // Extract session token from cookie
    let token = extract_session_token(&req);

    if let Some(token) = token {
        // Validate session against DB
        let valid = state
            .db
            .with_writer(|conn| {
                pi_kiosk_db::auth::validate_session(conn, &token)
            })
            .await
            .ok()
            .flatten();

        if valid.is_some() {
            return next.run(req).await;
        }
    }

    // Redirect to login for browser requests, 401 for API
    if wants_html(&req) {
        Redirect::to(LOGIN_PATH).into_response()
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

pub async fn optional_auth(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    if let Some(token) = extract_session_token(&req) {
        let valid = state
            .db
            .with_writer(|conn| pi_kiosk_db::auth::validate_session(conn, &token))
            .await
            .ok()
            .flatten();
        if valid.is_some() {
            return next.run(req).await;
        }
    }
    next.run(req).await
}

fn is_public_path(path: &str) -> bool {
    path == LOGIN_PATH
        || path == "/api/login"
        || path == "/api/logout"
        || path.starts_with("/pkg/")
        || path.starts_with("/style/")
        || path.starts_with("/favicon")
}

fn extract_session_token(req: &Request) -> Option<String> {
    let headers = req.headers();
    let cookie_header = headers.get(header::COOKIE)?;
    let cookie_str = cookie_header.to_str().ok()?;

    for cookie in cookie_str.split(';') {
        let cookie = cookie.trim();
        if let Some(rest) = cookie.strip_prefix(&format!("{}=", SESSION_COOKIE_NAME)) {
            return Some(rest.to_string());
        }
    }
    None
}

fn wants_html(req: &Request) -> bool {
    req.headers()
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("text/html"))
        .unwrap_or(false)
}

pub fn build_session_cookie(token: &str) -> String {
    format!(
        "{}={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=86400",
        SESSION_COOKIE_NAME, token
    )
}

pub fn build_logout_cookie() -> String {
    format!(
        "{}=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0",
        SESSION_COOKIE_NAME
    )
}
