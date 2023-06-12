use super::templates::{Base, Home, ReposCreate};
use crate::{app::App, config::CONFIG, models::RepoFile};
use axum::{
    body::Body,
    extract::{ConnectInfo, Json, Path, Query, State},
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
use git2::{
    BranchType::Local,
    DescribeOptions,
    ObjectType::{Commit, Tree},
    Oid, Repository,
};
use reqwest::StatusCode;
use serde_json::json;

pub async fn get_home(
    State(app): State<App>,
    TypedHeader(cookie): TypedHeader<Cookie>,
) -> impl IntoResponse {
    RenderHtml(
        "home",
        app.hbs,
        Base {
            signups_allowed: true,
            user_is_logged_in: cookie.get("logged_in").is_some(),
            nested: Home {
                name: "ibx34".to_string(),
            },
        },
    )
}

pub async fn get_repo(
    State(app): State<App>,
    TypedHeader(cookie): TypedHeader<Cookie>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let Ok(repo) = app.get_repo_by_on(&owner, &name).await else {
        return (StatusCode::NOT_FOUND, json!({}).to_string()).into_response();
    };

    let git_repo = match Repository::open_bare(&format!("./tests/{}/{}", owner, name)) {
        Ok(repo) => repo,
        Err(err) => {
            println!("{err:?}");
            return (StatusCode::NOT_FOUND, json!({"1":"!"}).to_string()).into_response();
        }
    };
    if let (Ok(head), Ok(requested_branch)) =
        (git_repo.head(), git_repo.find_branch("master", Local))
    {
        if let (Ok(tree), Ok(head)) = (requested_branch.get().peel_to_tree(), head.peel_to_commit())
        {
            if !tree.is_empty() {
                let author = head.author();
                let tree = tree
                    .iter()
                    .map(|e: git2::TreeEntry<'_>| {
                        let file = e.to_object(&git_repo).unwrap();
                        println!("1: {:?}", file.kind());
                        let file_size = if let Some(blob) = file.as_blob() {
                            println!("{:?}", blob.clone().into_object().peel_to_commit());
                            blob.content().to_vec().len()
                        } else {
                            0
                        };
                        return RepoFile {
                            filename: e.name().unwrap().to_string(),
                            hash: file.id().to_string(),
                            dir: file.as_tree().is_some(),
                        };
                    })
                    .collect::<Vec<RepoFile>>();

                return RenderHtml(
                    "repos/view",
                    app.hbs,
                    Base {
                        signups_allowed: true,
                        user_is_logged_in: cookie.get("logged_in").is_some(),
                        nested: json!({
                            "empty": false,
                            "id": repo.id.to_string(),
                            "name": repo.name,
                            "repo_link": format!("{}/{}", repo.owner_name, repo.name),
                            "owner": {
                                "id": repo.owner,
                                "name": repo.owner_name
                            },
                            "tree": tree,
                            "head": {
                                "author": author.name(),
                                "message": head.message(),
                                "hash": head.id().to_string()
                            }
                        }),
                    },
                )
                .into_response();
            }
        }
    }

    return RenderHtml(
        "repos/view",
        app.hbs,
        Base {
            signups_allowed: true,
            user_is_logged_in: cookie.get("logged_in").is_some(),
            nested: json!({
                "empty": true,
                "id": repo.id.to_string(),
                "name": repo.name,
                "repo_link": format!("{}/{}", repo.owner_name, repo.name),
                "owner": {
                    "id": repo.owner,
                    "name": repo.owner_name
                },
            }),
        },
    )
    .into_response();
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
            signups_allowed: true,
            user_is_logged_in: cookie.get("logged_in").is_some(),
            nested: ReposCreate { owner: owner_name },
        },
    )
    .into_response()
}
