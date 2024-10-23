use axum::routing::get_service;
use axum::{
    http::StatusCode,
    Router,
};
use database::Database;
use dotenvy::dotenv;
use parser::parse;
use std::env;
use std::path::PathBuf;
use tower_http::services::ServeDir;
pub(crate) mod generated {
    pub(crate) mod money_view;
}

use api::money_view_server::MoneyView;
use api::{
    BalanceResponse, Empty, Tag, TagResponse, TextRequest, TransactionPartnerResponse,
    TransactionResponse,
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
    // ... existing code ...

    async fn get_tags(&self, _request: Request<Empty>) -> Result<Response<TagResponse>, Status> {
        let tags: Vec<Tag> = self
            .db
            .get_tags()
            .await
            .map_err(to_tonic_error)?
            .into_iter()
            .map(|t| t.into())
            .collect();

        Ok(Response::new(TagResponse { tags: tags })) // Placeholder return
    }

    async fn set_tag(&self, request: Request<Tag>) -> Result<Response<Empty>, Status> {
        self.db
            .save_tag(request.into_inner().into())
            .await
            .map_err(to_tonic_error)?;
        self.db.update_tags().await.map_err(to_tonic_error)?;

        Ok(Response::new(Empty {})) // Placeholder return
    }
    async fn send_text_data(
        &self,
        request: Request<TextRequest>,
    ) -> Result<Response<TransactionResponse>, Status> {
        let mt940_string = request.into_inner().data;
        println!("Len: {}", mt940_string.len());

        let data = parse(mt940_string)
            .await
            .map_err(|e| Status::unknown(e.to_string()))?;
        self.db.save_all(data).await.map_err(to_tonic_error)?;

        let data = self
            .db
            .get_all_transactions()
            .await
            .map_err(|e| Status::new(tonic::Code::Aborted, e.to_string()))?;
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
        response.transaction_partners = partners;
        Ok(Response::new(response))
    }

    async fn get_partner_balance(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<BalanceResponse>, Status> {
        let mut balance_response = BalanceResponse::default();
        balance_response.expenses = self
            .db
            .get_partner_balance(false)
            .await
            .map_err(to_tonic_error)?;
        balance_response.total_expenses = balance_response
            .expenses
            .iter()
            .map(|partner| partner.balance)
            .sum();
        balance_response.income = self
            .db
            .get_partner_balance(true)
            .await
            .map_err(to_tonic_error)?;
        balance_response.total_income = balance_response
            .income
            .iter()
            .map(|partner| partner.balance)
            .sum();
        Ok(Response::new(balance_response))
    }

    async fn get_tag_balance(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<BalanceResponse>, Status> {
        let mut balance_response = BalanceResponse::default();
        balance_response.expenses = self
            .db
            .get_tag_balance(false)
            .await
            .map_err(to_tonic_error)?;
        balance_response.total_expenses = balance_response
            .expenses
            .iter()
            .map(|partner| partner.balance)
            .sum();
        balance_response.income = self
            .db
            .get_tag_balance(true)
            .await
            .map_err(to_tonic_error)?;
        balance_response.total_income = balance_response
            .income
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
    let web_addr = env::var("MONEY_VIEW_WEB_HOST").unwrap();
    let web_dir = PathBuf::from("./web");

    // Service für statische Dateien
    let static_service = get_service(ServeDir::new(web_dir.clone()))
        .handle_error(|_| async { StatusCode::INTERNAL_SERVER_ERROR });

    // Router definieren, um statische Dateien und die index.html für alle nicht gefundenen Routen auszuliefern
    let app = Router::new()
        .fallback_service(static_service);

    let host = env::var("MONEY_VIEW_DB_HOST").unwrap();
    let database = env::var("MONEY_VIEW_DB_NAME").unwrap();
    let user_name = env::var("MONEY_VIEW_USER").unwrap();
    let password = env::var("MONEY_VIEW_DB_PASSWD").unwrap();
    let namespace = env::var("MONEY_VIEW_DB_NAMESPACE").unwrap();

    let db = Database::new(host, user_name, password, namespace, database)
        .await
        .map_err(|e| Status::new(tonic::Code::Aborted, e.to_string()))?;
    db.init_db().await?;

    let money_view = MoneyViewServer { db };
    let money_view = api::money_view_server::MoneyViewServer::new(money_view);
    let money_view = tonic_web::enable(money_view);

    let reflection_1 = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(api::FILE_DESCRIPTOR_SET)
        .build_v1()?;
    let reflection_1a = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(api::FILE_DESCRIPTOR_SET)
        .build_v1alpha()?;

    // Tokio select um beide Server gleichzeitig laufen zu lassen
    tokio::select! {
        _ = async {
            // Starte den Axum HTTP Server
            println!("Axum listening on {}", &web_addr);
            let listener = tokio::net::TcpListener::bind(web_addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        } => {},

        _ = async {
            // Starte den Tonic gRPC Server
            println!("Tonic gRPC listening on {}", &addr);
            Server::builder()
            .accept_http1(true)
            .add_service(reflection_1)
            .add_service(reflection_1a)
            .add_service(money_view)
            .serve(addr)
            .await.unwrap();
        } => {},
    }

    Ok(())
}

mod parser;
pub(crate) type ShortResult<T> = Result<T, Box<dyn std::error::Error>>;