use proto::hello_world::{
    greeter_server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};
use slog::{warn, Drain, Logger};
use tonic::{transport::Server, Request, Response, Status};

mod middleware;
mod proto;
mod request_context;

#[derive(Default)]
pub struct MyGreeter {}

pub struct Name(String);

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let log = request.extensions().get::<Logger>().unwrap().to_owned();
        let name = request.into_inner().name;
        if name == "David Bowie" {
            warn!(log, "Dead man sending requests...");
        }
        let reply = HelloReply::from(format!("Hello {name}!"));
        let mut response = Response::new(reply);
        response.extensions_mut().insert(Name(name));
        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let greeter = MyGreeter::default();

    println!("GreeterServer listening on {}", addr);

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = slog::Logger::root(drain, slog::o!("cargo_pkg" => env!("CARGO_PKG_VERSION")));

    let layer = tower::ServiceBuilder::new().layer(middleware::LoggingLayer { log });

    Server::builder()
        .layer(layer)
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
