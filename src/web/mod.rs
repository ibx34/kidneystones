mod routes;
mod templates;

use axum::{
    response::IntoResponse,
    routing::{get, get_service, Router},
};
use routes::{get_create_repo, get_home};
use tower_http::services::ServeDir;

use crate::app::App;

pub fn register_web_routes() -> Router<App> {
    Router::new()
        .nest(
            "/repos",
            Router::new().route("/create", get(get_create_repo)),
        )
        .route("/", get(get_home))
}
