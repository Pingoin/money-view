use crate::api::Transaction;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

#[derive(Debug)]
pub(crate) struct Database {
    db: Surreal<Client>,
}

impl Database {
    pub(crate) async fn new(
        host: String,
        user_name: String,
        password: String,
        namespace: String,
        database: String,
    ) -> surrealdb::Result<Self> {
        let db = Surreal::new::<Ws>(host).await?;
        // Signin as a namespace, database, or root user
        let result = db
            .signin(Root {
                username: &user_name,
                password: &password,
            })
            .await?;

        dbg!(result);
        // Select a specific namespace / database
        db.use_ns(namespace).use_db(database).await?;
        Ok(Self { db })
    }

    pub(crate) async fn save_transaction(
        &self,
        transaction: &Transaction,
    ) -> surrealdb::Result<()> {
        let id = transaction.transaction_id.clone();
        let result: Option<Transaction> = self
            .db
            .create(("transaction", id))
            .content(transaction.clone())
            .await?;
        println!("{:#?}", result);
        Ok(())
    }

    pub(crate) async fn get_all_transactions(&self) -> surrealdb::Result<Vec<Transaction>> {
        let transactions: Vec<Transaction> = self.db.select("transaction").await.unwrap();
        Ok(transactions)
    }
}
