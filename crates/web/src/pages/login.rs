use leptos::*;
use leptos_router::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <div class="login-page">
            <div class="login-card">
                <h1>"Pi Kiosk"</h1>
                <p class="login-subtitle">"Sign in to manage your device"</p>
                <LoginForm/>
            </div>
        </div>
    }
}

#[component]
fn LoginForm() -> impl IntoView {
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(false);
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());

    let login = create_action(move |_: &()| {
        let username = username.get();
        let password = password.get();
        async move {
            set_loading.set(true);
            set_error.set(None);
            let result = login_server(username, password).await;
            set_loading.set(false);
            match result {
                Ok(true) => {
                    let navigate = use_navigate();
                    navigate("/", Default::default());
                }
                Ok(false) => {
                    set_error.set(Some("Invalid username or password".into()));
                }
                Err(e) => {
                    set_error.set(Some(format!("Error: {e}")));
                }
            }
        }
    });

    view! {
        <form class="login-form" on:submit=move |ev| {
            ev.prevent_default();
            login.dispatch(());
        }>
            <div class="form-field">
                <label for="username">"Username"</label>
                <input
                    type="text"
                    id="username"
                    name="username"
                    placeholder="admin"
                    required
                    on:input=move |ev| set_username.set(event_target_value(&ev))
                />
            </div>
            <div class="form-field">
                <label for="password">"Password"</label>
                <input
                    type="password"
                    id="password"
                    name="password"
                    placeholder="••••••••"
                    required
                    on:input=move |ev| set_password.set(event_target_value(&ev))
                />
            </div>
            {move || error.get().map(|e| view! {
                <div class="login-error">{e}</div>
            })}
            <button type="submit" class="login-button" disabled=move || loading.get()>
                {move || if loading.get() { "Signing in…" } else { "Sign in" }}
            </button>
        </form>
    }
}

#[server(Login, "/api")]
pub async fn login_server(
    username: String,
    password: String,
) -> Result<bool, ServerFnError> {
    use leptos_axum::extract;
    use std::sync::Arc;
    use crate::state::AppState;
    use crate::auth::build_session_cookie;

    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let user = state
        .db
        .with_writer(|conn| pi_kiosk_db::auth::authenticate(conn, &username, &password))
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    match user {
        Some(user) => {
            let session = state
                .db
                .with_writer(|conn| pi_kiosk_db::auth::create_session(conn, &user.username))
                .await
                .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

            let opts = expect_context::<leptos_axum::ResponseOptions>();
            if let Ok(val) = axum::http::HeaderValue::from_str(&build_session_cookie(&session.token)) {
                opts.insert_header(axum::http::header::SET_COOKIE, val);
            }

            Ok(true)
        }
        None => Ok(false),
    }
}

#[server(Logout, "/api")]
pub async fn logout_server() -> Result<(), ServerFnError> {
    use crate::auth::build_logout_cookie;

    let opts = expect_context::<leptos_axum::ResponseOptions>();
    if let Ok(val) = axum::http::HeaderValue::from_str(&build_logout_cookie()) {
        opts.insert_header(axum::http::header::SET_COOKIE, val);
    }

    Ok(())
}

#[server(HasUsers, "/api")]
pub async fn has_users() -> Result<bool, ServerFnError> {
    use leptos_axum::extract;
    use std::sync::Arc;
    use crate::state::AppState;

    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let count = state
        .db
        .with_writer(|conn| pi_kiosk_db::auth::user_count(conn))
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(count > 0)
}

#[server(CreateInitialUser, "/api")]
pub async fn create_initial_user(
    username: String,
    password: String,
) -> Result<(), ServerFnError> {
    use leptos_axum::extract;
    use std::sync::Arc;
    use crate::state::AppState;

    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let existing = state
        .db
        .with_writer(|conn| pi_kiosk_db::auth::user_count(conn))
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    if existing > 0 {
        return Err(ServerFnError::ServerError("Users already exist".into()));
    }

    state
        .db
        .with_writer(|conn| pi_kiosk_db::auth::create_user(conn, &username, &password))
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(())
}
