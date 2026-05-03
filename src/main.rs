use crate::{templates::*, user::*};
use axum::{
    Form, Router,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_login::{
    AuthManagerLayerBuilder, login_required,
    tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer, cookie::time::Duration},
};
use tower_http::services::ServeDir;
use tower_sessions_file_store::FileSessionStorage;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod templates;
mod user;

type AuthSession = axum_login::AuthSession<Backend>;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "character_sheet=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Session layer.
    let session_store = FileSessionStorage::new();
    let session_layer = SessionManagerLayer::new(session_store.clone())
        .with_expiry(Expiry::OnInactivity(Duration::seconds(60 * 60)));
    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60 * 60)),
    );

    // Auth service.
    let backend = Backend::default();
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let assets_path = std::env::current_dir().unwrap();

    let app = Router::new()
        .route("/", get(home))
        .route("/calculate", post(calculate))
        .route("/characters", get(characters))
        .route("/campaigns", get(campaigns))
        .route("/settings", get(settings))
        .route("/items", get(items))
        .route("/classes", get(classes))
        .route("/notes", get(notes))
        .route_layer(login_required!(Backend, login_url = "/login"))
        .route("/login", get(login_page).post(login))
        .route("/logout", get(logout))
        .layer(auth_layer)
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", assets_path.to_str().unwrap())),
        );

    info!("Starting server on port 3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    deletion_task.await;
}

async fn home(auth_session: AuthSession) -> impl IntoResponse {
    let name = auth_session.user.expect("Protected Page").username;
    let template = HomepageTemplate { username: name };
    HtmlTemplate(template).into_response()
}

async fn calculate(auth_session: AuthSession, body: String) -> impl IntoResponse {
    let args: Vec<&str> = body.split('=').collect();
    let val: i32 = args[1].parse().unwrap_or(0);

    let template = StatResponseTemplate {
        fields: vec![
            StatField {
                id: "strength-modifier".to_string(),
                value: (val - 10) / 2,
            },
            StatField {
                id: "acrobatics-modifier".to_string(),
                value: (val - 10) / 2 + 11,
            },
        ],
    };
    HtmlTemplate(template)
}

async fn characters(auth_session: AuthSession) -> impl IntoResponse {
    let name = auth_session.user.expect("Protected Page").username;
    let template = UnderConstructionTemplate { username: name };
    HtmlTemplate(template).into_response()
}

async fn campaigns(auth_session: AuthSession) -> impl IntoResponse {
    let name = auth_session.user.expect("Protected Page").username;
    let template = UnderConstructionTemplate { username: name };
    HtmlTemplate(template).into_response()
}

async fn settings(auth_session: AuthSession) -> impl IntoResponse {
    let name = auth_session.user.expect("Protected Page").username;
    let template = UnderConstructionTemplate { username: name };
    HtmlTemplate(template).into_response()
}

async fn items(auth_session: AuthSession) -> impl IntoResponse {
    let name = auth_session.user.expect("Protected Page").username;
    let template = UnderConstructionTemplate { username: name };
    HtmlTemplate(template).into_response()
}

async fn classes(auth_session: AuthSession) -> impl IntoResponse {
    let name = auth_session.user.expect("Protected Page").username;
    let template = UnderConstructionTemplate { username: name };
    HtmlTemplate(template).into_response()
}

async fn notes(auth_session: AuthSession) -> impl IntoResponse {
    let name = auth_session.user.expect("Protected Page").username;
    let template = UnderConstructionTemplate { username: name };
    HtmlTemplate(template).into_response()
}

async fn login_page(Query(NextUrl { next }): Query<NextUrl>) -> impl IntoResponse {
    let template = LoginTemplate { next };
    HtmlTemplate(template)
}

async fn login(mut auth_session: AuthSession, Form(creds): Form<Credentials>) -> Response {
    let user = match auth_session.authenticate(creds.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let login_url = if let Some(next) = creds.next {
                format!("/login?next={next}")
            } else {
                "/login".to_string()
            };

            info!("redirecting to login page");

            return Redirect::to(&login_url).into_response();
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    } else {
        info!("[{}]({}) - logged in", user.username, user.id);
    }

    if let Some(ref next) = creds.next {
        Redirect::to(next)
    } else {
        Redirect::to("/")
    }
    .into_response()
}

async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.logout().await {
        Ok(Some(user)) => {
            info!("[{}]({}) - logged out", user.username, user.id,);
            Redirect::to("/login").into_response()
        }
        Ok(None) => Redirect::to("/login").into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
