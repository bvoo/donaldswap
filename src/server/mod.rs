pub mod api;
pub mod ws;

use crate::config::ConfigManager;
use crate::state::AppState;
use crate::swapper::Swapper;
use axum::{Router, routing::get};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct ServerState {
    pub config_manager: Arc<ConfigManager>,
    pub app_state: Arc<AppState>,
    pub swapper: Arc<Swapper>,
}

pub fn create_app(
    config_manager: Arc<ConfigManager>,
    app_state: Arc<AppState>,
    swapper: Arc<Swapper>,
) -> Router {
    let state = ServerState {
        config_manager,
        app_state,
        swapper,
    };

    Router::new()
        .route("/ws", get(ws::ws_handler))
        .merge(api::create_api_router())
        .nest_service("/", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
