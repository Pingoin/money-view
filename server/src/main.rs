use database::Database;
use dotenvy::dotenv;
use parser::parse;
use std::env;

use api::money_view_server::MoneyView;
use api::{
    Empty, PartnerBalanceResponse, TextRequest, TransactionPartnerResponse, TransactionResponse,
};
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
        let mt940_string = request.into_inner().data;
        println!("Len: {}", mt940_string.len());

        let data = parse(mt940_string)
            .await
            .map_err(|e| Status::unknown(e.to_string()))?;
        self.db
            .save_all(data.1.clone(), data.0.clone())
            .await
            .map_err(to_tonic_error)?;
        let mut response = TransactionResponse::default();
        response.transactions = data
            .0
            .iter()
            .map(|t| t.clone().into_transaction())
            .collect();
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

    async fn get_all_transaction_partners(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<TransactionPartnerResponse>, Status> {
        let partners = self
            .db
            .get_all_transaction_partners()
            .await
            .map_err(|e| Status::new(tonic::Code::Aborted, e.to_string()))?;

        let mut response = TransactionPartnerResponse::default();
        response.transaction_partners = partners
            .iter()
            .map(|partner| partner.clone().into_partner())
            .collect();
        Ok(Response::new(response))
    }

    async fn get_partner_balance(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<PartnerBalanceResponse>, Status> {
        let mut balance_response = PartnerBalanceResponse::default();
        balance_response.partner_expenses = self
            .db
            .get_partner_balance(false)
            .await
            .map_err(to_tonic_error)?;
        balance_response.total_expenses = balance_response
            .partner_expenses
            .iter()
            .map(|partner| partner.balance)
            .sum();
        balance_response.partner_income = self
            .db
            .get_partner_balance(true)
            .await
            .map_err(to_tonic_error)?;
        balance_response.total_income = balance_response
            .partner_income
            .iter()
            .map(|partner| partner.balance)
            .sum();
        Ok(Response::new(balance_response))
    }
}

fn to_tonic_error<T>(err: T) -> Status
where
    T: ToString,
{
    Status::new(tonic::Code::Aborted, err.to_string())
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
