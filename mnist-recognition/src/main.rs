use axum::{
    extract::{Multipart, State},
    response::Json,
    routing::post,
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tokio::net::TcpListener;

mod inference;
mod preprocess;

use inference::MnistModel;

async fn predict_handler(
    State(model): State<Arc<MnistModel>>,
    mut multipart: Multipart,
) -> Json<serde_json::Value> {
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Ok(data) = field.bytes().await {
            match preprocess::prepare_image(&data) {
                Ok(pixels) => {
                    let model_clone = model.clone();
                    let digit = tokio::task::spawn_blocking(move || model_clone.predict(&pixels))
                        .await
                        .unwrap_or(None);
                    if let Some(d) = digit {
                        return Json(json!({ "digit": d }));
                    } else {
                        return Json(json!({ "error": "Prediction failed" }));
                    }
                }
                Err(e) => {
                    eprintln!("Preprocessing error: {}", e);
                    return Json(json!({ "error": "Image preprocessing failed" }));
                }
            }
        }
    }
    Json(json!({ "error": "No valid image uploaded" }))
}

#[tokio::main]
async fn main() {
    println!("🤖 Loading MNIST ONNX model with tract...");
    let model = match MnistModel::new() {
        Ok(m) => Arc::new(m),
        Err(e) => {
            eprintln!("Failed to load model: {}", e);
            eprintln!("Make sure models/mnist-8.onnx exists");
            return;
        }
    };
    println!("✅ Model loaded");

    let app = Router::new()
        .route("/predict", post(predict_handler))
        .with_state(model);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🌐 Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
