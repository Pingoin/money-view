use std::collections::HashMap;

use crate::api::{self, BalanceInformation, LineItem, Transaction, TransactionPartner};
use crate::ShortResult;
use chrono::{Duration, NaiveDate};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

const DEFAULT_TAG_ID: (&'static str, &'static str) = ("tag", "default");

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

    pub(crate) async fn init_db(&self) -> ShortResult<()> {
        let default: Option<Tag> = self.db.select(DEFAULT_TAG_ID).await?;
        if default.is_none() {
            let defaut_tag = Tag {
                id: Thing::from(DEFAULT_TAG_ID),
                name: String::from("Sonstige"),
                keywords: Vec::new(),
            };
            let _result: Option<Tag> = self.db.create(DEFAULT_TAG_ID).content(defaut_tag).await?;
        }
        Ok(())
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
    pub(crate) async fn get_all_transactions(&self) -> surrealdb::Result<Vec<TransactionRecord>> {
        let transactions: Vec<TransactionRecord> = self.db.select("transaction").await?;
        Ok(transactions)
    }

    pub(crate) async fn get_all_transaction_partners(
        &self,
    ) -> ShortResult<Vec<TransactionPartner>> {
        let partners: Vec<String> = Vec::new();
        Ok(partners
            .into_iter()
            .map(|p| TransactionPartner { name: p })
            .collect())
    }
    pub(crate) async fn save_all(&self, transactions: Vec<TransactionRecord>) -> ShortResult<()> {
        let mut transaction_count = 0u32;
        let tags = self.get_tag_map().await?;
        for mut transaction in transactions {
            transaction = transaction.update_tags(&tags);
            self.save_transaction(transaction).await?;
            transaction_count += 1;
        }
        println!("Inserted:\n {transaction_count} transactions");
        Ok(())
    }

    pub(crate) async fn get_partner_balance(
        &self,
        positive: bool,
    ) -> surrealdb::Result<Vec<BalanceInformation>> {
        const BASE_QUERY: &'static str ="Select math::Sum(total_amount) as balance,partner_name as name, count() as transaction_count from transaction ";
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

    pub(crate) async fn update_tags(&self) -> ShortResult<()> {
        let tags = HashMap::new();
        let mut transactions: Vec<TransactionRecord> = self.get_all_transactions().await?;
        let (send, recv) = tokio::sync::oneshot::channel();
        rayon::spawn(move || {
            let result = transactions
                .into_par_iter()
                .map(|t| t.update_tags(&tags))
                .collect();
            let _ = send.send(result);
        });
        transactions = recv.await?;

        self.save_all(transactions).await?;
        Ok(())
    }

    async fn get_tag_map(&self) -> ShortResult<HashMap<Thing, Vec<String>>> {
        let mut tag_map = HashMap::new();
        let tags: Vec<Tag> = self.get_tags().await?;
        for tag in tags {
            tag_map.insert(tag.id.clone(), tag.keywords.clone());
        }
        Ok(tag_map)
    }

    pub(crate) async fn get_tags(&self) -> ShortResult<Vec<Tag>> {
        let result: Vec<Tag> = self.db.select("tag").await?;
        Ok(result)
    }

    pub(crate) async fn save_tag(&self, tag: Tag) -> ShortResult<()> {
        let id = (tag.id.tb.clone(), tag.id.id.clone().to_raw());

        let result: Option<Tag> = self.db.select(id.clone()).await?;

        if let Some(_) = result {
            let _result: Option<Tag> = self.db.update(id.clone()).content(tag).await?;
        } else {
            let _result: Option<Tag> = self.db.create(id.clone()).content(tag).await?;
        }

        Ok(())
    }
    pub(crate) async fn get_tag_balance(
        &self,
        positive: bool,
    ) -> surrealdb::Result<Vec<BalanceInformation>> {
        const BASE_QUERY: &'static str ="select math::sum(line_items.amount) as balance, line_items.tag_id.name as name, count() as transaction_count from(select line_items from transaction split line_items) ";
        const POSITIVE: &'static str = "where line_items.amount>0.0 ";
        const NEGATIVE: &'static str = "where line_items.amount<0.0 ";
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
pub(crate) struct Tag {
    pub(crate) id: Thing,
    pub(crate) name: String,
    pub(crate) keywords: Vec<String>,
}

impl From<api::Tag> for Tag {
    fn from(value: api::Tag) -> Self {
        Self {
            id: Thing::from(("tag",value.id.as_str())),
            name: value.name,
            keywords: value.key_words,
        }
    }
}

impl Into<api::Tag> for Tag{
    fn into(self) -> api::Tag {
        api::Tag {
            id: self.id.id.to_raw(),
            name: self.name,
            key_words: self.keywords,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) struct TransactionRecord {
    pub(crate) id: Thing,            // Unique identifier for the transaction
    pub(crate) account_id: String,   // ID of the bank account where the transaction occurred
    pub(crate) date: NaiveDate,      // Date of the transaction (e.g., "2024-09-04")
    pub(crate) total_amount: f32,    // Total amount of the transaction
    pub(crate) partner_name: String, // Reference ID to the transaction partner
    pub(crate) line_items: Vec<LineItemRecord>, // List of line items within the transaction
    pub(crate) description: String,  // Description or memo of the transaction
    pub(crate) balance_after_transaction: f32, // Account balance after the transaction
}

impl Default for TransactionRecord {
    fn default() -> Self {
        Self {
            id: Thing::from(("transaction", "tmp")),
            account_id: Default::default(),
            date: Default::default(),
            total_amount: Default::default(),
            partner_name: Default::default(),
            line_items: Default::default(),
            description: Default::default(),
            balance_after_transaction: Default::default(),
        }
    }
}

impl Into<Transaction> for TransactionRecord {
    fn into(self) -> Transaction {
        Transaction {
            id: self.id.id.to_raw(),
            account_id: self.account_id,
            date: (self.date - NaiveDate::default()).num_days(),
            total_amount: self.total_amount,
            partner_name: self.partner_name,
            line_items: self
                .line_items
                .into_iter()
                .map(|line| line.into())
                .collect(),
            description: self.description,
            balance_after_transaction: self.balance_after_transaction,
        }
    }
}

impl From<Transaction> for TransactionRecord {
    fn from(value: Transaction) -> Self {
        TransactionRecord {
            id: Thing::from(("transaction".to_string(), value.id.clone())),
            account_id: value.account_id.clone(),
            date: NaiveDate::default() + Duration::days(value.date),
            total_amount: value.total_amount,
            partner_name: value.partner_name.clone(),
            line_items: value.line_items.into_iter().map(|l| l.into()).collect(),
            description: value.description.clone(),
            balance_after_transaction: value.balance_after_transaction,
        }
    }
}

impl TransactionRecord {
    fn update_tags(mut self, tag_keywords: &HashMap<Thing, Vec<String>>) -> Self {
        self.line_items = self
            .line_items
            .into_iter()
            .filter(|item| item.description != "".to_owned())
            .collect();

        let line_amount: f32 = self.line_items.iter().map(|item| item.amount).sum();

        let id = if let Some(id) = find_first_matching_id(self.description.as_str(), tag_keywords) {
            id
        } else if let Some(id) = find_first_matching_id(self.partner_name.as_str(), tag_keywords) {
            id
        } else {
            Thing::from(DEFAULT_TAG_ID)
        };

        let leave_item = LineItemRecord {
            description: "".to_string(),
            amount: self.total_amount - line_amount,
            tag_id: id,
        };

        self.line_items.push(leave_item);
        self
    }
}

fn find_first_matching_id<'a>(
    input: &str,
    keyword_map: &'a HashMap<Thing, Vec<String>>,
) -> Option<Thing> {
    for (id, keywords) in keyword_map {
        if keywords.iter().any(|keyword| input.contains(keyword)) {
            return Some(id.clone());
        }
    }
    None
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) struct LineItemRecord {
    description: String,
    amount: f32,
    tag_id: Thing,
}

impl From<LineItem> for LineItemRecord {
    fn from(value: LineItem) -> Self {
        Self {
            description: value.description.clone(),
            amount: value.amount,
            tag_id: Thing::from(("tag".to_string(), value.tag_id.clone())),
        }
    }
}

impl Into<LineItem> for LineItemRecord {
    fn into(self) -> LineItem {
        LineItem {
            description: self.description,
            amount: self.amount,
            tag_id: self.tag_id.id.to_raw(),
        }
    }
}
