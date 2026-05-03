use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;

pub struct HtmlTemplate<T>(pub T);

#[derive(Debug, Deserialize)]
pub struct NextUrl {
    pub next: Option<String>,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub next: Option<String>,
}

#[derive(Template)]
#[template(path = "under-construction.html")]
pub struct UnderConstructionTemplate {
    pub username: String,
}

#[derive(Template)]
#[template(path = "homepage.html")]
pub struct HomepageTemplate {
    pub username: String,
}

pub struct StatField {
    pub id: String,
    pub value: i32,
}

#[derive(Template)]
#[template(path = "stat_response.html")]
pub struct StatResponseTemplate {
    pub fields: Vec<StatField>,
}

/// Allows us to convert Askama HTML templates into valid HTML for axum to serve in the response.
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        // Attempt to render the template with askama
        match self.0.render() {
            // If we're able to successfully parse and aggregate the template, serve it
            Ok(html) => Html(html).into_response(),
            // If we're not, return an error or some bit of fallback HTML
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}
