mod app;
mod config;
mod db;
mod models;
mod routes;

use crate::{
    app::App,
    routes::{
        git::{info_refs, recieve_pack, upload_pack},
        repos::create_repo,
    },
};

use anyhow::Result;
use axum::{
    // middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

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
    //.layer(from_fn_with_state(app_state.clone(), session_middleware));

    let app = Router::new()
        // git remote add <name> http://127.0.0.1:8080/ibx34/<repo-name>[.git]
        .route("/:user/:repo/git-upload-pack", post(upload_pack))
        .route("/:user/:repo/git-receive-pack", post(recieve_pack))
        .route("/:user/:repo/info/refs", get(info_refs))
        .nest("/repos", repositories)
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
