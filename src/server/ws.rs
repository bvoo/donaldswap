use crate::server::ServerState;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::Response,
};
use futures::{SinkExt, StreamExt};
use tracing::info;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state.app_state))
}

async fn handle_socket(socket: WebSocket, app_state: std::sync::Arc<crate::state::AppState>) {
    let (mut tx, mut rx) = socket.split();
    let mut receiver = app_state.broadcaster.subscribe();

    let initial_state = app_state.get_state().await;
    let msg = serde_json::to_string(&initial_state).unwrap_or_default();
    if tx.send(Message::Text(msg)).await.is_err() {
        return;
    }

    let send_task = async move {
        while let Ok(state) = receiver.recv().await {
            let msg = serde_json::to_string(&state).unwrap_or_default();
            if tx.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    };

    let recv_task = async move {
        while let Some(msg) = rx.next().await {
            if msg.is_err() {
                break;
            }
        }
    };

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    info!("WebSocket disconnected");
}
