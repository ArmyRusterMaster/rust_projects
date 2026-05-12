use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = auth_service::http::build_router();
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
