use dotenvy::dotenv;
use std::env;

use api::money_view_server::MoneyView;
use api::{Empty, TextRequest, Transaction, TransactionResponse};
use couch_rs::document::TypedCouchDocument;
use couch_rs::types::find::FindQuery;
use couch_rs::CouchDocument;
use tonic::{transport::Server, Request, Response, Status};

pub mod api {
    use std::borrow::Cow;

    use couch_rs::document::TypedCouchDocument;

    tonic::include_proto!("money_view");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("reflection");

    impl TypedCouchDocument for Transaction {
        fn get_id(&self) -> Cow<str> {
            let id = self.id.clone();
            Cow::Owned(id)
        }

        fn get_rev(&self) -> Cow<str> {
            let rev = self.rev.clone();
            Cow::Owned(rev)
        }

        fn set_rev(&mut self, rev: &str) {
            self.rev = rev.to_string();
        }

        fn set_id(&mut self, id: &str) {
            self.id = id.to_string();
        }

        fn merge_ids(&mut self, other: &Self) {
            self.id = other.id.clone();
            self.rev = other.rev.clone();
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct MoneyViewServer {}

#[tonic::async_trait]
impl MoneyView for MoneyViewServer {
    async fn send_text_data(
        &self,
        request: Request<TextRequest>,
    ) -> Result<Response<TransactionResponse>, Status> {
        let db_host = env::var("MONEY_VIEW_DB_HOST").unwrap();
        let db_name = env::var("MONEY_VIEW_DB_NAME").unwrap();
        let db_user = env::var("MONEY_VIEW_USER").unwrap();
        let db_password = env::var("MONEY_VIEW_DB_PASSWD").unwrap();

        let client = couch_rs::Client::new(db_host.as_str(), db_user.as_str(), db_password.as_str()).unwrap();
        let db = client.db(db_name.as_str()).await.unwrap();
        let find_all = FindQuery::find_all();
        let docs = db.find_raw(&find_all).await.unwrap();
        let mut doc = Transaction::default();
        doc.set_id(format!("{}:{}", "bla", &uuid::Uuid::new_v4().to_string()).as_str());
        db.save(&mut doc).await.map_err(|e| {
            println!("{:?}", e);
            Status::new(tonic::Code::Unknown, format!("{:?}", e))
        })?;
        println!("{:?}", docs);
        println!("Request: {:?}", request.into_inner().data);

        let data = db.get_all::<Transaction>().await.map_err(|e| {
            println!("{:?}", e);
            Status::new(tonic::Code::Unknown, format!("{:?}", e))
        })?;

        let data = data.rows;
        let mut response = TransactionResponse::default();
        response.transactions = data;
        Ok(Response::new(response))
    }

    async fn get_all_transactions(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<TransactionResponse>, Status> {

        let db_host = env::var("MONEY_VIEW_DB_HOST").unwrap();
        let db_name = env::var("MONEY_VIEW_DB_NAME").unwrap();
        let db_user = env::var("MONEY_VIEW_USER").unwrap();
        let db_password = env::var("MONEY_VIEW_DB_PASSWD").unwrap();


        let client = couch_rs::Client::new(db_host.as_str(), db_user.as_str(), db_password.as_str()).unwrap();
        let db = client.db(db_name.as_str()).await.unwrap();

        let data = db.get_all::<Transaction>().await.map_err(|e| {
            println!("{:?}", e);
            Status::new(tonic::Code::Unknown, format!("{:?}", e))
        })?;

        let data = data.rows;
        let mut response = TransactionResponse::default();
        response.transactions = data;
        Ok(Response::new(response))
    }
}

#[derive(serde::Deserialize, serde::Serialize, CouchDocument)]
struct Test {
    pub _id: String,
    pub _rev: String,
    pub data: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let addr = "0.0.0.0:50051".parse()?;
    let money_view = MoneyViewServer {};
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
