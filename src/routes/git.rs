use std::{io::Bytes, process::Stdio};

use axum::{
    extract::{BodyStream, Path, Query},
    http::header::{HeaderMap, CONTENT_TYPE},
    response::IntoResponse,
    routing::{get, post},
};
use futures_util::StreamExt;
use git2::Repository;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{io::AsyncWriteExt, process::Command};

const MAX_SIZE: usize = 524288000;

pub fn strip_dot_git_from_repo_name(n: String) -> Result<String, impl IntoResponse> {
    let repo_name_split = n.split(".");
    let split_count = repo_name_split.to_owned().count() > 2;
    // This should only be two parts so we are gonna return an error if it isnt
    if split_count {
        // TODO: add gooder errors
        return Err((StatusCode::BAD_REQUEST, json!({}).to_string()));
    }
    let repo_name = if let Some(last) = repo_name_split.to_owned().last() {
        if split_count && last != "git" {
            println!("ye");
            return Err((StatusCode::BAD_REQUEST, json!({}).to_string()));
        }
        // TODO: errors should be handle
        repo_name_split
            .collect::<Vec<&str>>()
            .first()
            .unwrap()
            .to_string()
    } else {
        n
    };
    Ok(repo_name)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitSmartHttpRequest {
    pub service: Option<String>,
}
pub async fn info_refs(
    Query(smart_request): Query<GitSmartHttpRequest>,
    Path((user, repo)): Path<(String, String)>,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        format!(
            "application/x-{}-advertisement",
            smart_request.service.to_owned().unwrap()
        )
        .parse()
        .unwrap(),
    );
    let repo_name = strip_dot_git_from_repo_name(repo)
        .map_err(|e| return e.into_response())
        .unwrap();
    let repo_path = &format!("/home/alfredo/kidney-stones/tests/{}/{}", user, repo_name);
    println!("Path3: {repo_path:?}");
    match Repository::open_bare(repo_path) {
        Ok(repo) => repo,
        Err(_) => return (StatusCode::NOT_FOUND, headers, json!({}).to_string()).into_response(),
    };
    let mut buffer = Vec::new();
    let service = smart_request.service.to_owned().unwrap();
    let __pkt_line = format!("# service={}\n", service);

    let _pkt_line = (__pkt_line.len() + 4) as i64;
    let mut pkt_line = String::from_utf8(
        (&base16::encode_byte(_pkt_line.try_into().unwrap(), base16::EncodeLower)).to_vec(),
    )
    .unwrap();

    if pkt_line.len() % 4 != 0 {
        pkt_line = (0..4 - pkt_line.len() % 4).map(|_| "0").collect::<String>() + &pkt_line;
    }

    pkt_line = pkt_line + &__pkt_line;
    // 11
    std::io::Write::write_all(&mut buffer, pkt_line.as_bytes()).unwrap();
    std::io::Write::write_all(&mut buffer, "0000".as_bytes()).unwrap();

    // meow2222
    let child = Command::new("git")
        .current_dir(repo_path)
        .args(&[
            if service == "git-recieve-pack" {
                "receive-pack"
            } else {
                "upload-pack"
            },
            "--stateless-rpc",
            "--advertise-refs",
            ".",
        ])
        .output()
        .await
        .expect("failed to spawn");

    std::io::Write::write_all(&mut buffer, &child.stdout).unwrap();
    (StatusCode::OK, headers, buffer).into_response()
}

pub async fn recieve_pack(
    Path((user, repo)): Path<(String, String)>,
    mut payload: BodyStream,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    let repo_name = strip_dot_git_from_repo_name(repo)
        .map_err(|e| return e.into_response())
        .unwrap();
    let repo_path = &format!("/home/alfredo/kidney-stones/tests/{}/{}", user, repo_name);
    println!("Path2: {repo_path:?}");
    match Repository::open_bare(repo_path) {
        Ok(repo) => repo,
        Err(_) => return (StatusCode::NOT_FOUND, headers, json!({}).to_string()).into_response(),
    };
    let mut body = Vec::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.unwrap();
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            println!("TOO BIG");
            return (StatusCode::BAD_REQUEST, headers, json!({}).to_string()).into_response();
        }
        body.extend_from_slice(&chunk);
    }

    let mut buffer = Vec::new();
    let mut child = Command::new("git")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .env("SSH_ORIGINAL_COMMAND", "receive-pack")
        .env("GIT_TRACE", "1")
        .env("GIT_TRACE_PACKET", "1")
        .env("GIT_CURL_VERBOSE", "1")
        .args(&["receive-pack", "--stateless-rpc", repo_path])
        .spawn()
        .expect("failed to spawn");
    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    stdin.write_all(&body).await.unwrap();
    let output = child
        .wait_with_output()
        .await
        .expect("Failed to read stdout");
    std::io::Write::write_all(&mut buffer, &output.stdout).unwrap();
    headers.insert(
        CONTENT_TYPE,
        format!("application/x-{}-result", "git-receive-pack")
            .parse()
            .unwrap(),
    );

    (StatusCode::OK, headers, buffer).into_response()
}

pub async fn upload_pack(
    Path((user, repo)): Path<(String, String)>,
    mut payload: BodyStream,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    let repo_name = strip_dot_git_from_repo_name(repo)
        .map_err(|e| return e.into_response())
        .unwrap();
    let repo_path = &format!("/home/alfredo/kidney-stones/tests/{}/{}", user, repo_name);
    println!("Path1: {repo_path:?}");
    match Repository::open_bare(repo_path) {
        Ok(repo) => repo,
        Err(_) => return (StatusCode::NOT_FOUND, headers, json!({}).to_string()).into_response(),
    };

    let mut body = Vec::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.unwrap();
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return (StatusCode::BAD_REQUEST, headers, json!({}).to_string()).into_response();
        }
        body.extend_from_slice(&chunk);
    }

    let mut buffer = Vec::new();

    let mut child = Command::new("git")
        .current_dir(repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(&["upload-pack", "--stateless-rpc", "."])
        .spawn()
        .expect("failed to spawn");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    stdin.write(&body).await.unwrap();

    let output = child
        .wait_with_output()
        .await
        .expect("Failed to read stdout");
    std::io::Write::write_all(&mut buffer, &output.stdout).unwrap();

    headers.insert(
        CONTENT_TYPE,
        format!("application/x-{}-result", "git-upload-pack")
            .parse()
            .unwrap(),
    );
    (StatusCode::OK, headers, buffer).into_response()
}
