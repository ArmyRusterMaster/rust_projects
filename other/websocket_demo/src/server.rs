use tungstenite::accept;
use std::net::TcpListener;
use tungstenite::Message;
use tracing::{info, error, debug};

pub fn start_server() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    info!("Server started on ws://127.0.0.1:8080");

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            tokio::spawn(async move {
                match accept(stream) {
                    Ok(mut websocket) => {
                        info!("Client connected");

                        loop {
                            match websocket.read_message() {
                                Ok(msg) => {
                    debug!("Received message: {:?}", msg);
                    match msg {
                        Message::Text(text) => {
                            info!("Received text message: {}", text);
                            if text == "ping" {
                                websocket.write_message(Message::Text("pong".to_string())).unwrap();
                                info!("Sent pong response");
                            } else if text.starts_with("echo:") {
                                let echo_msg = &text[5..];
                                let response = format!("Echo: {}", echo_msg);
                                websocket.write_message(Message::Text(response.clone())).unwrap();
                                info!("Sent echo response: {}", response);
                            } else {
                                websocket.write_message(
                    Message::Text(format!("Unknown command: {}", text))
                ).unwrap();
            }
        }
        Message::Ping(_) => {
            websocket.write_message(Message::Pong(vec![])).unwrap();
            info!("Responded to ping with pong frame");
        }
        Message::Close(_) => {
            info!("Client requested connection close");
            break;
        }
        _ => debug!("Ignored message type: {:?}", msg),
            }
        }
                Ok(Message::Close(frame)) => {
                    info!("Client closed connection: {:?}", frame);
                    break;
                }
                Err(e) => {
                    error!("Error reading message: {}", e);
                    break;
                }
            }
        }

        info!("Client disconnected");
    },
    Err(e) => error!("Failed to accept WebSocket connection: {}", e),
}
            });
        }
    }
}

