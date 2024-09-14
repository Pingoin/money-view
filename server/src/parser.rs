use crate::api::{Transaction, TransactionPartner};
use chrono::NaiveTime;
use lazy_static::lazy_static;
use mt940::{parse_mt940, sanitizers::sanitize, DebitOrCredit, ExtDebitOrCredit, Message};
use rayon::prelude::*;
use regex::Regex;
use rust_decimal::Decimal;
type ShortResult<T> = Result<T, Box<dyn std::error::Error>>;

lazy_static! {
    static ref FIELD_KEY_PARTIAL_ERAZER: Regex =
        Regex::new(r"\$(2([1-9])|3([3-9])|6([1-9]))").unwrap();
    static ref FIELD_KEY_COMPLETE_ERAZER: Regex = Regex::new(r"\?\d{2}").unwrap();
    static ref USAGE_KEY_RENAMER: Regex =
        Regex::new(r"([A-Z])(REF|RED|REN|VWZ|BAN|IC)(\+|: )").unwrap();
}

pub async fn parse(input: String) -> ShortResult<Vec<Transaction>> {
    let input = process_input(input).await?;
    let input = sanitize(input.as_str());
    let input_parsed = parse_mt940(&input)?;

    let messages = process_messages(input_parsed).await?;

    Ok(messages)
}

async fn process_messages(input: Vec<Message>) -> ShortResult<Vec<Transaction>> {
    let (send, recv) = tokio::sync::oneshot::channel();
    rayon::spawn(move || {
        let result: Vec<Transaction> = input
            .par_iter()
            .map(|entry| {
                //println!("{:#?}", &entry);
                let mut balance: f32 = entry.opening_balance.amount.try_into().unwrap();
                if entry.opening_balance.debit_credit_indicator == DebitOrCredit::Debit {
                    balance *= -1.0;
                }
                let account_id = entry.account_id.clone();
                let transactions: Vec<Transaction> = entry
                    .statement_lines
                    .iter()
                    .map(|line| {
                        let mut transaction = Transaction::default();
                        transaction.total_amount =
                            parse_amount(line.amount.clone(), &line.ext_debit_credit_indicator)
                                .unwrap_or_default();
                        if let Some(date) = line.entry_date {
                            transaction.date = date.and_time(NaiveTime::MIN).and_utc().timestamp();
                        }
                        balance += transaction.total_amount;
                        transaction.balance_after_transaction = balance;
                        transaction.account_id = account_id.clone();
                        if let Some(mut info) = line.information_to_account_owner.clone() {
                            info = info.replace("\n", "");
                            info = FIELD_KEY_COMPLETE_ERAZER
                                .replace_all(info.as_str(), "")
                                .into_owned();
                            info = USAGE_KEY_RENAMER
                                .replace_all(&info, "\n$1$2: ")
                                .into_owned();
                            #[derive(Default)]
                            struct LineContent {
                                kref: Option<String>,
                                eref: Option<String>,
                                svwz: Option<String>,
                                mref: Option<String>,
                                iban: Option<String>,
                                cren: Option<String>,
                            }

                            let mut line_content = LineContent::default();

                            info.lines().for_each(|line| {
                                let ln: Vec<&str> = line.split(": ").collect();

                                match ln[0] {
                                    "KREF" => line_content.kref = Some(ln[1].to_string()),
                                    "EREF" => line_content.eref = Some(ln[1].to_string()),
                                    "SVWZ" => line_content.svwz = Some(ln[1].to_string()),
                                    "MREF" => line_content.mref = Some(ln[1].to_string()),
                                    "IBAN" => line_content.iban = Some(ln[1].to_string()),
                                    "CREN" => line_content.cren = Some(ln[1].to_string()),
                                    _ => {}
                                }
                            });

                            if let Some(svwz) = line_content.svwz {
                                transaction.description = svwz.clone();
                            }
                            if let Some(kref) = line_content.kref {
                                transaction.transaction_id = kref;
                            } else if let Some(eref) = line_content.eref {
                                transaction.transaction_id = eref;
                            } else {
                                transaction.transaction_id = generate_id(
                                    transaction.total_amount,
                                    transaction.balance_after_transaction,
                                    transaction.description.as_str(),
                                );
                            }
                            let mut partner = TransactionPartner::default();
                            if let Some(cren) = line_content.cren {
                                partner.name = format!("{}", cren);
                            }

                            if let Some(mref) = line_content.mref {
                                partner.partner_id = format!("{}", mref);
                            } else if let Some(iban) = line_content.iban {
                                partner.partner_id = format!("{}", iban);
                            }
                            transaction.partner = Some(partner);
                        };
                        transaction
                    })
                    .collect();
                transactions
            })
            .flatten()
            .collect();

        let _ = send.send(result);
    });
    Ok(recv.await?)
}

use sha2::{Digest, Sha256};

fn generate_id(f1: f32, f2: f32, s1: &str) -> String {
    // Kombiniere die Werte zu einem einzigen String
    let combined = format!("{}{}{}", f1, f2, s1);

    // Erstelle eine neue SHA256-Instanz
    let mut hasher = Sha256::new();

    // Füge die kombinierten Bytes hinzu
    hasher.update(combined.as_bytes());

    // Berechne den Hash und konvertiere ihn in einen hexadezimalen String
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn parse_amount(amount: Decimal, debit: &ExtDebitOrCredit) -> ShortResult<f32> {
    let val: f32 = amount.try_into()?;
    let val = val
        * match debit {
            mt940::ExtDebitOrCredit::Debit => -1.0,
            mt940::ExtDebitOrCredit::Credit => 1.0,
            mt940::ExtDebitOrCredit::ReverseDebit => -1.0,
            mt940::ExtDebitOrCredit::ReverseCredit => 1.0,
        };

    Ok(val)
}

fn move_inserted_to_end(line: String) -> String {
    // Suche nach dem Start ($32) und dem Ende ($60) des Einschubs
    if let Some(start) = line.find("$32") {
        if let Some(end) = line.find("$60") {
            // Der Einschub ohne die Markierungen $32 und $60
            let snippet = &line[start + 3..end]; // +3, um das "$32" zu überspringen, bis zum Ende "$60"
                                                 // Entferne den Einschub inklusive der Markierungen aus der Zeile
            let mut new_line = line
                .replacen(&line[start..=end + 2], "", 1)
                .trim_end()
                .to_string();
            // Den Einschub ans Ende der Zeile verschieben mit "CREN+"
            new_line.push_str(&format!(" CREN+{}", snippet));
            return new_line;
        }
    }
    // Wenn kein Einschub vorhanden ist, die Zeile unverändert zurückgeben
    line.to_string()
}
fn remove_between_eref_and_iban(line: String) -> String {
    // Suche nach "EREF:" und "IBAN:"
    if let Some(start) = line.find("EREF:") {
        if let Some(end) = line.find("IBAN:") {
            // Entferne alles von "EREF:" bis einschließlich "IBAN:"
            let cleaned_line = format!("{}{}", &line[..start], &line[end..]);
            return cleaned_line;
        } else if let Some(end) = line.find("CREN+") {
            // Wenn "IBAN:" nicht gefunden wird, suche nach "CREN+"
            let cleaned_line = format!("{}{}", &line[..start], &line[end..]);
            return cleaned_line;
        }
    }
    // Wenn "EREF:" oder "IBAN:" nicht vorhanden ist, die Zeile unverändert zurückgeben
    line.to_string()
}

fn short_process(input: String) -> String {
    input.replace("?34992", "").replace("?", "$")
}

fn remove_numbers(input: String) -> String {
    FIELD_KEY_PARTIAL_ERAZER
        .replace_all(input.as_str(), "")
        .into_owned()
}

async fn process_input(input: String) -> ShortResult<String> {
    // Verarbeite jede Zeile und sammle die Ergebnisse
    let (send, recv) = tokio::sync::oneshot::channel();
    let input = input.replace("\n?", "?");

    rayon::spawn(move || {
        let processed_lines: Vec<String> = input
            .par_lines()
            .map(|line| line.to_string())
            .map(short_process)
            .map(remove_numbers)
            .map(move_inserted_to_end)
            .map(remove_between_eref_and_iban)
            .collect();
        let _ = send.send(processed_lines.join("\n"));
    });
    // Kombiniere die verarbeiteten Zeilen zu einem einzigen String
    Ok(recv.await?)
}
