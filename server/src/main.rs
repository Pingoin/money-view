use api::money_view_server::MoneyView;
use api::{TextRequest, TransactionResponse};
use tonic::{transport::Server, Request, Response, Status};

pub mod api {
    tonic::include_proto!("money_view");
}

#[derive(Debug, PartialEq, Eq)]
struct MoneyViewServer {}

#[tonic::async_trait]
impl MoneyView for MoneyViewServer {
    async fn send_text_data(
        &self,
        request: Request<TextRequest>,
    ) -> Result<Response<TransactionResponse>, Status> {
        println!("Request: {:?}", request.into_inner().data);
        Ok(Response::new(TransactionResponse::default()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let money_view = MoneyViewServer {};
    let money_view = api::money_view_server::MoneyViewServer::new(money_view);
    let money_view = tonic_web::enable(money_view);
    Server::builder()
        .accept_http1(true)
        .add_service(money_view)
        .serve(addr)
        .await?;
    Ok(())
}
