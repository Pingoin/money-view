use tonic::{transport::Server, Request, Response, Status};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};

pub mod hello_world {
    tonic::include_proto!("hello"); // The string specified here must match the proto package name
}

#[derive(Debug, PartialEq, Eq,Default)]
struct GreetServer{

}

#[tonic::async_trait]
impl Greeter for GreetServer {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<HelloReply>, Status> { // Return an instance of type HelloReply
        println!("Got a request: {:?}", request);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let greeter = GreetServer::default();
    let greeter=GreeterServer::new(greeter);
    let greeter= tonic_web::enable(greeter);
    Server::builder()
    .accept_http1(true)
        .add_service(greeter)
        .serve(addr)
        .await?;

    Ok(())
}