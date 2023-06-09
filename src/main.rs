mod app;
mod config;
mod db;
mod models;
mod routes;
mod web;

use crate::{
    app::App,
    routes::{
        git::{info_refs, recieve_pack, upload_pack},
        repos::create_repo,
    },
    web::register_web_routes,
};

use anyhow::Result;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
    // middleware::from_fn_with_state,
    routing::{get, get_service, post},
    Router,
};
use axum_extra::body::AsyncReadBody;
use reqwest::StatusCode;
use std::net::SocketAddr;
use tower_http::{services::ServeDir, trace::TraceLayer};

async fn web() -> impl IntoResponse {
    let Ok(file) = tokio::fs::File::open("/home/alfredo/kidney-stones/web/dist/index.html").await else {
        panic!();
    };
    (AsyncReadBody::new(file))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .pretty()
        .init();

    let app = App::init().await?;
    // throw the result away, we dont care.
    let _ = app.create_account("ibx34", "haha").await;

    let repositories = Router::new().route("/create", post(create_repo));
    let api = Router::new()
        .route("/:user/:repo/git-upload-pack", post(upload_pack))
        .route("/:user/:repo/git-receive-pack", post(recieve_pack))
        .route("/:user/:repo/info/refs", get(info_refs))
        .nest("/repos", repositories);
    //.layer(from_fn_with_state(app_state.clone(), session_middleware));
    let web_routes = register_web_routes();
    let app = Router::new()
        .nest_service(
            "/assets",
            get_service(ServeDir::new("/home/alfredo/kidney-stones/assets")),
        )
        .nest_service(
            "/dist",
            get_service(ServeDir::new("/home/alfredo/kidney-stones/web/dist/")),
        )
        // git remote add <name> http://127.0.0.1:8080/ibx34/<repo-name>[.git]
        .nest("/api", api)
        .nest("/", web_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(app);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Starting on: {addr:?}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;
    println!("Started");
    Ok(())
}
