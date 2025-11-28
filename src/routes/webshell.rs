use axum::{
    Router,
    routing::get,
    extract::{
        ws::{WebSocket, Message},
        ws,
        Path,
    },
    response::IntoResponse,
};

use crate::state::AppState;

pub fn webshell_routes() -> Router<AppState> {
    Router::new().route("/webshell/:session_id", get(ws_handler))
}

async fn ws_handler(
    ws: ws::WebSocketUpgrade,
    Path(_session_id): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let _ = socket.send(Message::Text("Connected to fake shell".into())).await;

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(txt) => {
                let _ = socket.send(Message::Text(format!("echo: {}", txt))).await;
            }
            Message::Binary(bin) => {
                let _ = socket.send(Message::Binary(bin)).await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}
