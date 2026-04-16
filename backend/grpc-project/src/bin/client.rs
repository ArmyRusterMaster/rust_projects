pub mod hello {
    tonic::include_proto!("hello");
}
use hello::HelloRequest;
use hello::greeter_client::GreeterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(HelloRequest {
        name: "Termux User".into(),
    });
    let response = client.say_hello(request).await?;
    println!("Ответ сервера: {:?}", response.into_inner().message);
    Ok(())
}
