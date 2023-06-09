use super::templates::{Base, Home, ReposCreate, ReposCreateDefaults};
use crate::{app::App, config::CONFIG};
use axum::{
    body::Body,
    extract::{ConnectInfo, Json, Path as PathExtractor, Query, State},
    headers::{authorization::Bearer, Authorization, Cookie},
    http::{
        header::{HeaderMap, CONTENT_TYPE},
        Request,
    },
    response::IntoResponse,
    routing::{get, post},
    TypedHeader,
};
use axum_template::RenderHtml;
use reqwest::StatusCode;
use serde_json::json;

pub async fn get_home(State(app): State<App>) -> impl IntoResponse {
    let person = Home {
        name: "ibx34".to_string(),
    };

    RenderHtml("home", app.hbs, person)
}

pub async fn get_create_repo(
    State(app): State<App>,
    TypedHeader(cookie): TypedHeader<Cookie>,
) -> impl IntoResponse {
    let owner_name = if let Some(session) = cookie.get("session") {
        let Ok((_, user)) = app.get_session(session).await else {
            return (StatusCode::INTERNAL_SERVER_ERROR, json!({"msg":"failed to get user session"}).to_string()).into_response();
        };
        user.name.to_owned()
    } else {
        "ghost".to_string()
    };

    RenderHtml(
        "repos/create",
        app.hbs,
        Base {
            signups_allowed: false,
            nested: ReposCreate {
                user_is_logged_in: cookie.get("logged_in").is_some(),
                defaults: ReposCreateDefaults { owner: owner_name },
            },
        },
    )
    .into_response()
}
