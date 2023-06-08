use std::{io::Bytes, net::SocketAddr, path::Path, process::Stdio};

use crate::{app::App, config::CONFIG};

use super::git::strip_dot_git_from_repo_name;
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
use futures_util::StreamExt;
use git2::{Repository, Status};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{io::AsyncWriteExt, process::Command};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRepoData {
    pub name: String,
    // This field really shouldn't be used. When authorized, the owner comes from the session owner's name
    // And when unauthorized, it's put under ghost. For now, while there arent users, then this field should
    // be touched
    pub owner: Option<String>,
    pub ttl: Option<i64>,
    pub private: bool,
}

pub async fn create_repo(
    State(app): State<App>,
    // This will be needed later:tm:
    TypedHeader(cookie): TypedHeader<Cookie>,
    ConnectInfo(_client_addr): ConnectInfo<SocketAddr>,
    Json(create_repo_data): Json<CreateRepoData>,
) -> impl IntoResponse {
    let owner = if let Some(session) = cookie.get("session") {
        // get session -> user
        let Ok((session, user)) = app.get_session(session).await else {
            return (StatusCode::INTERNAL_SERVER_ERROR, json!({}).to_string()).into_response();
        };
        (user.name.to_owned(), user.id)
    } else {
        // The user is creating an anonymous repo. Some of the checks can be done here!
        if create_repo_data.private || create_repo_data.ttl.is_none() {
            return (StatusCode::FORBIDDEN, json!({}).to_string()).into_response();
        }
        (
            String::from("anonymous"),
            CONFIG.anonymous_repos_user.to_owned(),
        )
    };
    // Repo paths should NOT end in .git. They can when being created or call upon
    let stripped_repo_name = strip_dot_git_from_repo_name(create_repo_data.name)
        .map_err(|e| return e.into_response())
        .unwrap();
    let path = format!("./tests/{}/{}", owner.0, stripped_repo_name);
    let bare_git_repo_path = Path::new(&path);
    if bare_git_repo_path.exists() {
        // Repo exists,c reate better errors omg
        return (StatusCode::FORBIDDEN, json!({}).to_string()).into_response();
    }
    if Repository::init_bare(bare_git_repo_path).is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, json!({}).to_string()).into_response();
    }
    app.create_repo(&stripped_repo_name, owner.1, &owner.0)
        .await
        .unwrap();
    (StatusCode::OK, json!({}).to_string()).into_response()
}
