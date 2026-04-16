use tonic::{transport::Server, Request, Response, Status};
pub mod hello {
    tonic::include_proto!("hello");
}
use hello::greeter_server::{Greeter, GreeterServer};
use hello::{HelloResponse, HelloRequest};

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloResponse>, Status> {
        let reply = HelloResponse {
            message: format!("Привет, {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();
    println!("Сервер запущен на {}", addr);
    Server::builder().add_service(GreeterServer::new(greeter)).serve(addr).await?;
    Ok(())
}

