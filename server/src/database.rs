use crate::api::{BalanceInformation, LineItem, Transaction, TransactionPartner};
use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
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
        transaction: TransactionRecord,
    ) -> surrealdb::Result<()> {
        let result: std::prelude::v1::Option<TransactionRecord> = self
            .db
            .select((
                transaction.id.tb.clone(),
                transaction.id.id.clone().to_raw(),
            ))
            .await?;
        if let Some(_) = result {
            let _result: std::prelude::v1::Option<TransactionRecord> = self
                .db
                .update((
                    transaction.id.tb.clone(),
                    transaction.id.id.clone().to_raw(),
                ))
                .content(transaction)
                .await?;
        } else {
            let _result: std::prelude::v1::Option<TransactionRecord> = self
                .db
                .create((
                    transaction.id.tb.clone(),
                    transaction.id.id.clone().to_raw(),
                ))
                .content(transaction)
                .await?;
        }
        Ok(())
    }
    pub(crate) async fn get_all_transactions(&self) -> surrealdb::Result<Vec<Transaction>> {
        let transactions: Vec<TransactionRecord> = self.db.select("transaction").await?;
        let transactions: Vec<Transaction> = transactions
            .iter()
            .map(|t| t.clone().into_transaction())
            .collect();
        Ok(transactions)
    }

    pub(crate) async fn save_transaction_partner(
        &self,
        partner: TransactionPartnerRecord,
    ) -> surrealdb::Result<()> {
        let ressource = (partner.id.tb.clone(), partner.id.id.clone().to_raw());

        let result: std::prelude::v1::Option<TransactionPartnerRecord> =
            self.db.select(ressource.clone()).await?;

        if let Some(_) = result {
            let _result: std::prelude::v1::Option<TransactionPartnerRecord> = self
                .db
                .update((partner.id.tb.clone(), partner.id.id.clone().to_raw()))
                .merge(partner)
                .await?;
        } else {
            let _result: std::prelude::v1::Option<TransactionPartnerRecord> = self
                .db
                .create((partner.id.tb.clone(), partner.id.id.clone().to_raw()))
                .content(partner)
                .await?;
        }
        Ok(())
    }

    pub(crate) async fn get_all_transaction_partners(
        &self,
    ) -> surrealdb::Result<Vec<TransactionPartnerRecord>> {
        let partners: Vec<TransactionPartnerRecord> = self.db.select("partner").await?;
        Ok(partners)
    }
    pub(crate) async fn save_all(
        &self,
        partners: Vec<TransactionPartnerRecord>,
        transactions: Vec<TransactionRecord>,
    ) -> surrealdb::Result<()> {
        let mut partner_count = 0u32;
        let mut transaction_count = 0u32;
        for partner in partners {
            self.save_transaction_partner(partner).await?;
            partner_count += 1;
        }
        for transaction in transactions {
            self.save_transaction(transaction).await?;
            transaction_count += 1;
        }
        println!("Inserted:\n {transaction_count} transactions\n {partner_count} partners");
        Ok(())
    }

    pub(crate) async fn get_partner_balance(
        &self,
        positive: bool,
    ) -> surrealdb::Result<Vec<BalanceInformation>> {
        const BASE_QUERY: &'static str ="Select math::Sum(total_amount) as balance,partner_id.name as name, count() as transaction_count from transaction ";
        const POSITIVE: &'static str = "where total_amount>0.0 ";
        const NEGATIVE: &'static str = "where total_amount<0.0 ";
        const GROUP: &'static str = "group name;";
        let result: Vec<BalanceInformation> = self
            .db
            .query(format!(
                "{}{}{}",
                BASE_QUERY,
                if positive { POSITIVE } else { NEGATIVE },
                GROUP
            ))
            .await?
            .take(0)?;
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) struct TransactionRecord {
    pub(crate) id: Thing,                 // Unique identifier for the transaction
    pub(crate) account_id: String,        // ID of the bank account where the transaction occurred
    pub(crate) date: NaiveDate,           // Date of the transaction (e.g., "2024-09-04")
    pub(crate) total_amount: f32,         // Total amount of the transaction
    pub(crate) partner_id: Thing,         // Reference ID to the transaction partner
    pub(crate) line_items: Vec<LineItem>, // List of line items within the transaction
    pub(crate) description: String,       // Description or memo of the transaction
    pub(crate) balance_after_transaction: f32, // Account balance after the transaction
}

impl Default for TransactionRecord {
    fn default() -> Self {
        Self {
            id: Thing::from(("transaction", "tmp")),
            account_id: Default::default(),
            date: Default::default(),
            total_amount: Default::default(),
            partner_id: Thing::from(("partner", "tmp")),
            line_items: Default::default(),
            description: Default::default(),
            balance_after_transaction: Default::default(),
        }
    }
}

impl TransactionRecord {
    #[allow(unused)]
    pub(crate) fn from_transaction(value: &Transaction) -> Self {
        TransactionRecord {
            id: Thing::from(("transaction".to_string(), value.id.clone())),
            account_id: value.account_id.clone(),
            date: NaiveDate::default() + Duration::days(value.date),
            total_amount: value.total_amount,
            partner_id: Thing::from(("partner".to_string(), value.partner_id.clone())),
            line_items: value.line_items.clone(),
            description: value.description.clone(),
            balance_after_transaction: value.balance_after_transaction,
        }
    }
    pub fn into_transaction(self) -> Transaction {
        Transaction {
            id: self.id.id.to_raw(),
            account_id: self.account_id,
            date: (self.date - NaiveDate::default()).num_days(),
            total_amount: self.total_amount,
            partner_id: self.partner_id.id.to_raw(),
            line_items: self.line_items,
            description: self.description,
            balance_after_transaction: self.balance_after_transaction,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct TransactionPartnerRecord {
    pub(crate) id: Thing,
    pub(crate) name: String,
}

impl Default for TransactionPartnerRecord {
    fn default() -> Self {
        Self {
            id: Thing::from(("partner", "no")),
            name: Default::default(),
        }
    }
}

impl TransactionPartnerRecord {
    #[allow(unused)]
    pub(crate) fn from_partner(value: &TransactionPartner) -> Self {
        TransactionPartnerRecord {
            id: Thing::from(("partner".to_string(), value.id.clone())),
            name: value.name.clone(),
        }
    }

    pub fn into_partner(self) -> TransactionPartner {
        TransactionPartner {
            id: self.id.id.to_raw(),
            name: self.name,
        }
    }
}
