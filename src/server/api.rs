use crate::server::ServerState;
use crate::windows;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tracing::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct WindowInfoResponse {
    pub hwnd: isize,
    pub title: String,
    pub exe_name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub games: Option<Vec<crate::config::GameConfig>>,
    pub min_swap_minutes: Option<u32>,
    pub max_swap_minutes: Option<u32>,
    pub auto_swap_enabled: Option<bool>,
    pub hide_next_swap: Option<bool>,
    pub obs_ws_host: Option<String>,
    pub obs_ws_port: Option<u16>,
    pub obs_ws_password: Option<Option<String>>,
}

#[derive(Debug, Serialize)]
pub struct StateResponse {
    pub current_game: Option<String>,
    pub current_exe: Option<String>,
    pub last_swap_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_swap_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_paused: bool,
    pub swap_count: u64,
    pub time_since_swap_seconds: Option<i64>,
    pub time_until_swap_seconds: Option<i64>,
    pub history: Vec<crate::state::SwapHistoryItem>,
    #[serde(default)]
    pub total_times: HashMap<String, u64>,
}

pub fn create_api_router() -> Router<ServerState> {
    Router::new()
        .route("/api/config", get(get_config).put(update_config))
        .route("/api/windows", get(get_windows))
        .route("/api/state", get(get_state))
        .route("/api/swap", post(force_swap))
        .route("/api/pause", post(pause))
        .route("/api/resume", post(resume))
}

async fn get_config(State(state): State<ServerState>) -> impl IntoResponse {
    let config = state.config_manager.get().await;
    Json(config)
}

async fn update_config(
    State(state): State<ServerState>,
    Json(req): Json<UpdateConfigRequest>,
) -> impl IntoResponse {
    let config = state
        .config_manager
        .update(|c| {
            if let Some(games) = req.games {
                c.games = games;
            }
            if let Some(min) = req.min_swap_minutes {
                c.min_swap_minutes = min;
            }
            if let Some(max) = req.max_swap_minutes {
                c.max_swap_minutes = max;
            }
            if let Some(enabled) = req.auto_swap_enabled {
                c.auto_swap_enabled = enabled;
            }
            if let Some(hide) = req.hide_next_swap {
                c.hide_next_swap = hide;
            }
            if let Some(host) = req.obs_ws_host {
                c.obs_ws_host = host;
            }
            if let Some(port) = req.obs_ws_port {
                c.obs_ws_port = port;
            }
            if let Some(pass) = req.obs_ws_password {
                c.obs_ws_password = pass;
            }
        })
        .await;

    match config {
        Ok(c) => (StatusCode::OK, Json(c)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_windows() -> impl IntoResponse {
    match windows::enumerate_windows() {
        Ok(windows) => {
            let response: Vec<WindowInfoResponse> = windows
                .into_iter()
                .map(|w| WindowInfoResponse {
                    hwnd: w.hwnd,
                    title: w.title,
                    exe_name: w.exe_name,
                })
                .collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Vec::<WindowInfoResponse>::new()),
        )
            .into_response(),
    }
}

async fn get_state(State(state): State<ServerState>) -> impl IntoResponse {
    let swap_state = state.app_state.get_state().await;
    let now = chrono::Utc::now();

    let time_since_swap_seconds = swap_state.last_swap_at.map(|t| (now - t).num_seconds());
    let time_until_swap_seconds = swap_state.next_swap_at.map(|t| (t - now).num_seconds());

    Json(StateResponse {
        current_game: swap_state.current_game,
        current_exe: swap_state.current_exe,
        last_swap_at: swap_state.last_swap_at,
        next_swap_at: swap_state.next_swap_at,
        is_paused: swap_state.is_paused,
        swap_count: swap_state.swap_count,
        time_since_swap_seconds,
        time_until_swap_seconds,
        history: swap_state.history,
        total_times: swap_state.total_times,
    })
}

async fn force_swap(State(state): State<ServerState>) -> impl IntoResponse {
    info!("Force swap requested");
    match state.swapper.force_swap().await {
        Ok(()) => {
            info!("Force swap completed successfully");
            let swap_state = state.app_state.get_state().await;
            (StatusCode::OK, Json(swap_state)).into_response()
        }
        Err(e) => {
            warn!("Force swap failed: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn pause(State(state): State<ServerState>) -> impl IntoResponse {
    state.app_state.update_state(|s| s.is_paused = true).await;
    let swap_state = state.app_state.get_state().await;
    Json(swap_state)
}

async fn resume(State(state): State<ServerState>) -> impl IntoResponse {
    state.app_state.update_state(|s| s.is_paused = false).await;
    let swap_state = state.app_state.get_state().await;
    Json(swap_state)
}
