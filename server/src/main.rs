use database::Database;
use dotenvy::dotenv;
use parser::parse;
use std::env;

use api::money_view_server::MoneyView;
use api::{Empty, TextRequest, Transaction, TransactionResponse};
use tonic::{transport::Server, Request, Response, Status};

pub(crate) mod api;
pub(crate) mod database;

#[derive(Debug)]
struct MoneyViewServer {
    db: Database,
}

#[tonic::async_trait]
impl MoneyView for MoneyViewServer {
    async fn send_text_data(
        &self,
        request: Request<TextRequest>,
    ) -> Result<Response<TransactionResponse>, Status> {
        //let find_all = FindQuery::find_all();
        let mut doc = Transaction::default();
        doc.transaction_id = format!("{}:{}", "bla", &uuid::Uuid::new_v4().to_string());
        self.db
            .save_transaction(&doc)
            .await
            .map_err(|e| Status::new(tonic::Code::Aborted, e.to_string()))?;
        let mt940_string = request.into_inner().data;

        let data = parse(mt940_string)
            .await
            .map_err(|e| Status::unknown(e.to_string()))?;
        let mut response = TransactionResponse::default();
        response.transactions = data;
        Ok(Response::new(response))
    }

    async fn get_all_transactions(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<TransactionResponse>, Status> {
        let transactions = self
            .db
            .get_all_transactions()
            .await
            .map_err(|e| Status::new(tonic::Code::Aborted, e.to_string()))?;

        let mut response = TransactionResponse::default();
        response.transactions = transactions;
        Ok(Response::new(response))
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Test {
    pub _id: String,
    pub _rev: String,
    pub data: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let addr = "0.0.0.0:50051".parse()?;
    let host = env::var("MONEY_VIEW_DB_HOST").unwrap();
    let database = env::var("MONEY_VIEW_DB_NAME").unwrap();
    let user_name = env::var("MONEY_VIEW_USER").unwrap();
    let password = env::var("MONEY_VIEW_DB_PASSWD").unwrap();
    let namespace = env::var("MONEY_VIEW_DB_NAMESPACE").unwrap();

    let db = Database::new(host, user_name, password, namespace, database)
        .await
        .map_err(|e| Status::new(tonic::Code::Aborted, e.to_string()))?;

    let money_view = MoneyViewServer { db };
    let money_view = api::money_view_server::MoneyViewServer::new(money_view);
    let money_view = tonic_web::enable(money_view);

    let reflection_1 = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(api::FILE_DESCRIPTOR_SET)
        .build_v1()?;
    let reflection_1a = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(api::FILE_DESCRIPTOR_SET)
        .build_v1alpha()?;

    Server::builder()
        .accept_http1(true)
        .add_service(reflection_1)
        .add_service(reflection_1a)
        .add_service(money_view)
        .serve(addr)
        .await?;
    Ok(())
}

mod parser;
