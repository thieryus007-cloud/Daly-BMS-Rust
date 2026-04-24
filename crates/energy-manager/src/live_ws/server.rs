/// WebSocket live server — broadcasts LiveEvent JSON to all connected clients.
/// Endpoint: WS /live
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::types::LiveEvent;

#[derive(Clone)]
struct WsState {
    tx: broadcast::Sender<LiveEvent>,
}

pub async fn serve(bind: &str, live_tx: broadcast::Sender<LiveEvent>) {
    let state = WsState { tx: live_tx };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/live", get(ws_handler))
        .route("/health", get(health_handler))
        .with_state(state)
        .layer(cors);

    let addr: SocketAddr = bind.parse().unwrap_or_else(|_| "0.0.0.0:8081".parse().unwrap());
    info!("Live WebSocket server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_handler() -> &'static str {
    "energy-manager ok"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<WsState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state.tx))
}

async fn handle_socket(mut socket: WebSocket, tx: broadcast::Sender<LiveEvent>) {
    let mut rx = tx.subscribe();
    loop {
        match rx.recv().await {
            Ok(event) => {
                let json = match serde_json::to_string(&event) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if socket.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("WebSocket client lagged {n} events");
            }
            Err(_) => break,
        }
    }
}
