use tungstenite::{connect, Message};
use url::Url;
use tracing::{info, error, debug};

pub async fn start_client() {
    info!("Attempting to connect to server...");

    match connect(Url::parse("ws://127.0.0.1:8080").unwrap()) {
        Ok((mut websocket, _)) => {
            info!("Successfully connected to server");

            // Ping-pong test
            info!("Sending ping...");
            websocket.write_message(Message::Text("ping".to_string())).unwrap();

            match websocket.read_message() {
                Ok(msg) => match msg {
                    Message::Text(text) => {
                info!("Received response: {}", text);
                assert_eq!(text, "pong", "Expected 'pong' response");
            }
            _ => error!("Unexpected message type in ping-pong response"),
        },
        Err(e) => error!("Error reading ping response: {}", e),
    }

    // Echo test
    let test_message = "Hello from client with tracing!";
    info!("Sending echo message: {}", test_message);
    websocket.write_message(Message::Text(format!("echo:{}", test_message))).unwrap();

    match websocket.read_message() {
        Ok(msg) => match msg {
            Message::Text(text) => {
                info!("Received echo response: {}", text);
                assert!(text.contains(test_message), "Echo response doesn't contain original message");
            }
            _ => error!("Unexpected message type in echo response"),
        },
        Err(e) => error!("Error reading echo response: {}", e),
    }

    // Test unknown command
    info!("Testing unknown command...");
    websocket.write_message(Message::Text("unknown".to_string())).unwrap();

    match websocket.read_message() {
        Ok(msg) => match msg {
            Message::Text(text) => info!("Received error response: {}", text),
            _ => error!("Unexpected message type for unknown command"),
        },
        Err(e) => error!("Error reading unknown command response: {}", e),
    }

    info!("Client test completed successfully");
},
Err(e) => error!("Failed to connect to server: {}", e),
    }
}

