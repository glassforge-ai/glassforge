//! WebSocket at GET /api/v1/ws: forwards EventBus events to connected clients.

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State,
        WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/ws", get(ws_handler))
}

pub async fn ws_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    // Subscribe to the broadcast channel (best-effort delivery for UI).
    let mut bus_rx = state.event_bus.subscribe();

    // Forward events from EventBus to the WebSocket client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = bus_rx.recv().await {
            match serde_json::to_string(&event) {
                Ok(json) => {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("failed to serialize event for WebSocket: {}", e);
                }
            }
        }
    });

    // Consume incoming messages so the socket stays open.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(_msg)) = receiver.next().await {
            // Could handle ping/pong or client commands here.
        }
    });

    // Wait for either task to finish, then cleanly abort the other.
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
        }
    }

    tracing::debug!("WebSocket connection closed");
}
