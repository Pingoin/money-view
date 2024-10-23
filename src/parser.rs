use std::collections::HashMap;

use crate::{database::TransactionRecord, ShortResult};
use itertools::Itertools;
use lazy_static::lazy_static;
use mt940::{parse_mt940, sanitizers::sanitize, DebitOrCredit, ExtDebitOrCredit, Message};
use rayon::prelude::*;
use regex::Regex;
use rust_decimal::Decimal;

lazy_static! {
    static ref FIELD_KEY_PARTIAL_ERAZER: Regex =
        Regex::new(r"\$(2([1-9])|3([3-9])|6([1-9]))").unwrap();
    static ref FIELD_KEY_COMPLETE_ERAZER: Regex = Regex::new(r"\?\d{2}").unwrap();
    static ref USAGE_KEY_RENAMER: Regex =
        Regex::new(r"([A-Z])(REF|RED|REN|VWZ|BAN|IC)(\+|: )").unwrap();
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
        } else {
            return line.replace("$32", "CREN+");
        }
    }
    // Wenn kein Einschub vorhanden ist, die Zeile unverändert zurückgeben
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

pub async fn parse(input: String) -> ShortResult<Vec<TransactionRecord>> {
    let input = pre_parser(input).await?;
    println!("preparse: {:?}", input.len());
    let res = parse_mt940(&input)?;
    println!("messages: {:?}", res.len());
    Ok(parse_messages(res).await?)
}

pub async fn pre_parser(input: String) -> ShortResult<String> {
    let input = input
        .replace("\r\n", "\n")
        .replace("\n?", "?")
        .replace("\n-", "");
    let (send, recv) = tokio::sync::oneshot::channel();
    rayon::spawn(move || {
        let _ = send.send(
            input
                .par_lines()
                .map(|line| line.to_string())
                .map(short_process)
                .map(move_inserted_to_end)
                .map(remove_numbers)
                .map(|line| {
                    if line.starts_with(":86:") {
                        parse_86_line(line.clone()).unwrap_or(line)
                    } else {
                        line
                    }
                })
                .map(|s| sanitize(&s))
                .collect::<Vec<String>>()
                .join("\n"),
        );
    });
    Ok(recv.await?)
}

async fn parse_messages(input: Vec<Message>) -> ShortResult<Vec<TransactionRecord>> {
    let (send, recv) = tokio::sync::oneshot::channel();
    rayon::spawn(move || {
        let result: Vec<TransactionRecord> = input
            .par_iter()
            .map(process_single_message)
            .flatten()
            .collect();
        let _ = send.send(result);
    });
    let mut result = recv.await?;
    result = result.into_iter().unique_by(|t| t.id.clone()).collect();
    Ok(result)
}
fn process_single_message(input: &Message) -> Vec<TransactionRecord> {
    let mut balance: f32 = input.opening_balance.amount.try_into().unwrap_or_default();
    if input.opening_balance.debit_credit_indicator == DebitOrCredit::Debit {
        balance *= -1.0;
    }
    let account_id = input.account_id.clone();

    let result: Vec<TransactionRecord> = input
        .statement_lines
        .iter()
        .map(|line| {
            let mut transaction = TransactionRecord::default();
            transaction.total_amount =
                parse_amount(line.amount.clone(), &line.ext_debit_credit_indicator)
                    .unwrap_or_default();
            if let Some(date) = line.entry_date {
                transaction.date = date;
            }
            balance += transaction.total_amount;
            transaction.balance_after_transaction = balance;
            transaction.account_id = account_id.clone();
            if let Some(info) = line.information_to_account_owner.clone() {
                let info: Vec<&str> = info.split("?").collect();

                transaction.id = surrealdb::sql::Thing::from((
                    "transaction",
                    format!(
                        "{}-{}-{}",
                        transaction.date,
                        transaction.total_amount,
                        info.get(1).unwrap_or(&"")
                    )
                    .as_str(),
                ));
                transaction.description = info.get(4).unwrap_or(&"").to_string();
                transaction.partner_name = info.get(3).unwrap_or(&"").to_string();
            }
            transaction
        })
        .collect();
    result
}

fn parse_86_line(line: String) -> ShortResult<String> {
    let mut transaction_id = "".to_string();
    let mut partner_id = "".to_string();
    let mut partner_name = "".to_string();
    let mut transaction_description = "".to_string();
    let line = line.replace(": ", "+");
    let parsed_line = parse_string(line.as_str());

    if let Some(svwz) = parsed_line.get(&"SVWZ".to_string()) {
        transaction_description = svwz.clone();
    }

    if let Some(kref) = parsed_line.get(&"KREF".to_string()) {
        transaction_id = kref.clone();
    } else if let Some(eref) = parsed_line.get("EREF") {
        transaction_id = eref.clone();
    }

    if let Some(cren) = parsed_line.get("CREN") {
        partner_name = format!("{}", cren);
    } else if line.contains("00Abschluss") {
        partner_name = "VR-BANK UCKERMARK-RANDOW".to_string();
    }

    if let Some(mref) = parsed_line.get("MREF") {
        partner_id = format!("{}", mref);
    } else if let Some(iban) = parsed_line.get("IBAN") {
        partner_id = format!("{}", iban);
    } else if partner_name != "".to_string() {
        partner_id = partner_name.clone();
    }

    if partner_id == "OFFLINE".to_string() {
        partner_id = partner_name.clone();
    }

    Ok(format!(
        ":86:999?{}?{}?{}?{}",
        norm(transaction_id),
        norm(partner_id),
        norm(partner_name),
        norm(transaction_description)
    ))
}

fn parse_string(input: &str) -> HashMap<String, String> {
    let mut results: HashMap<String, String> = HashMap::new();

    // Regex zum Erkennen von Schlüsselwörtern: Bis zu 4 Großbuchstaben gefolgt von einem +
    let keyword_regex = Regex::new(r"([A-Z]{1,4})\+").unwrap();

    // Wir gehen den gesamten String durch und suchen nach den Keywords
    let mut pos = 0;

    while let Some(cap) = keyword_regex.captures(&input[pos..]) {
        let keyword = &cap[1]; // Das Schlüsselwort ohne das "+"
        let keyword_end = pos + cap.get(0).unwrap().end(); // Position nach dem "+" im String

        // Finde die Position des nächsten Keywords oder das Ende des Strings
        let next_keyword_start = keyword_regex
            .find(&input[keyword_end..])
            .map_or(input.len(), |m| keyword_end + m.start());

        // Den Inhalt zwischen dem aktuellen Keyword und dem nächsten sammeln
        let value = input[keyword_end..next_keyword_start].trim().to_string();

        // Speichere das Keyword und den Inhalt in der HashMap
        results.entry(keyword.to_string()).or_insert(value);

        // Setze die Position auf das nächste Keyword oder das Ende des Strings
        pos = next_keyword_start;
    }

    results
}

fn norm(input: String) -> String {
    // Regex für aufeinanderfolgende Leerzeichen
    let regex = regex::Regex::new(r"\s+").unwrap();
    // Ersetze durch ein einzelnes Leerzeichen
    regex.replace_all(input.as_str(), " ").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse() {
        let input: String = ":20:STARTUMS
:25:15091704/3000185000
:28C:0
:60F:D240716EUR757,99
:61:2407160716DR104,50NDDTKREF+
:86:105?00Basislastschrift?10931?20EREF+390773481601010055
?21KREF+2024071094085283092700?22000000000023?23MREF+0035-5250
?24CRED+DE71ZZZ00001448453?25SVWZ+Drente, Konstantin 06 
?2624 EREF: 390773481601010055?27 MREF: 0035-5250 CRED: DE71
?28ZZZ00001448453 IBAN: DE5952?290604100006418015 BIC: GENOD
?32Ev. Kirchengemeinde Torgelo?33w?34992?60EF1EK1
:61:2407160716DR19,99NDDTKREF+
:86:105?00Basislastschrift?10931?20EREF+5QYE0CZOZDIZ5NA2
?21KREF+2024071598289479092700?22000000001011
?23MREF+H.WtTcdq9awruT-bPNDJ.k?24K0x+oooL
?25CRED+DE94ZZZ00000561653?26SVWZ+028-2069999-2465952 AM
?27ZN Mktp DE 5QYE0CZOZDIZ5NA2?28 EREF: 5QYE0CZOZDIZ5NA2 MRE
?29F: H.WtTcdq9awruT-bPNDJ.kK0?32AMAZON PAYMENTS EUROPE S.C.?33A.
?34992?60x+oooL CRED: DE94ZZZ0000056
?611653 IBAN: DE87300308801908?62262006 BIC: TUBDDEDD
:61:2407160716DR13,99NDDTKREF+
:86:105?00Basislastschrift?10931?20EREF+3H715HGANVXTCYIP
?21KREF+2024071598289479092700?22000000003203
?23MREF+H.WtTcdq9awruT-bPNDJ.k?24K0x+oooL
?25CRED+DE94ZZZ00000561653?26SVWZ+028-3725317-9286760 AM
?27ZN Mktp DE 3H715HGANVXTCYIP?28 EREF: 3H715HGANVXTCYIP MRE
?29F: H.WtTcdq9awruT-bPNDJ.kK0?32AMAZON PAYMENTS EUROPE S.C.?33A.
?34992?60x+oooL CRED: DE94ZZZ0000056
?611653 IBAN: DE87300308801908?62262006 BIC: TUBDDEDD
:61:2407160716DR48,32NDDTKREF+
:86:105?00Basislastschrift?10931?20EREF+1035612656016
?21KREF+2024071598689042092700?22000000000026
?23MREF+5TX2224W7MDA6?24CRED+LU96ZZZ000000000000000?250058
?26SVWZ+1035612656016/. SHELL ?27DEUTSCHLAND GmbH, Ihr Einka
?28uf bei SHELL DEUTSCHLAND Gm?29bH EREF: 1035612656016 MREF
?32PayPal Europe S.a.r.l. et C?33ie S.C.A?34992
?60: 5TX2224W7MDA6 CRED: LU96Z?61ZZ0000000000000000058 IBAN:
?62 LU89751000135104200E BIC: ?63PPLXLUL2
:61:2407160716DR30,84NDDTKREF+
:86:106?00Basislastschrift?10931?20EREF+5440879830874315072410?215322
?22KREF+2024071699701654092700?23000000003376?24MREF+OFFLINE
?25CRED+DE53ZZZ00001600000?26PURP+IDCP
?27SVWZ+ALDI SAGT DANKE 29 044?28/Torgelow/DE               
?29     15.07.2024 um 10:53:22?32ALDI GmbH + Co. KG JARMEN?34011
?60 Uhr 54408798/308743/ECTL/ ?61     15091704/3000185000/1/
?621225
:62F:D240716EUR975,63"
            .to_string();

        let result = parse(input).await.unwrap();
        dbg!(result);
    }
}
