use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io;

use crate::paysterk::charge::{ChargeAuthorizationRequest, ChargeAuthorizationResponse};
use crate::paysterk::client::PaystackClient;
use reqwest::Method;

#[derive(Serialize, Debug)]
pub struct InitializeTransactionRequest {
    pub email: String,
    pub amount: u32,
    pub callback_url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct InitializeTransactionResponse {
    pub status: bool,
    pub message: String,
    pub data: Value,
}

#[derive(Serialize, Debug)]
pub struct VerifyTransactionRequest {
    pub reference: String,
}

#[derive(Deserialize, Debug)]
pub struct TransactionResponse {
    pub status: bool,
    pub message: String,
    pub data: TxData,
}

#[derive(Deserialize, Debug)]
pub struct TxData {
    pub id: i64,
    pub domain: Option<String>,
    pub status: Option<String>,
    pub reference: Option<String>,
    pub receipt_number: Option<String>,
    pub amount: i64,
    pub message: Option<String>,
    pub gateway_response: Option<String>,
    pub paid_at: Option<String>,
    pub created_at: Option<String>,
    pub channel: Option<String>,
    pub currency: Option<String>,
    pub ip_address: Option<String>,
    pub metadata: Option<Value>,
    pub tx_log: Option<TxLog>,
    pub fees: Option<i64>,
    pub fees_split: Option<String>,
    pub authorization: TxAuthorization,
    pub customer: TxCustomer,
    pub plan: Option<Value>,
    pub split: Option<Value>,
    pub order_id: Option<String>,
    pub requested_amount: i64,
    pub pos_transaction_data: Option<String>,
    pub source: Option<Txsource>,
    pub fees_breakdown: Option<String>,
    pub connect: Option<String>,
    pub transaction_date: Option<String>,
    pub plan_object: Option<Value>,
    pub sub_account: Option<Value>,
}

#[derive(Deserialize, Debug)]
pub struct Txsource {
    pub r#type: Option<String>,
    pub source: Option<String>,
    pub identifier: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TxLog {
    pub start_time: u32,
    pub time_spent: u32,
    pub attempts: u32,
    pub errors: u32,
    pub success: bool,
    pub mobile: bool,
    pub input: Vec<String>,
    pub tx_hist: Vec<TxLogHistory>,
}

#[derive(Deserialize, Debug)]
pub struct TxLogHistory {
    pub r#type: String,
    pub message: String,
    pub time: u32,
}

#[derive(Deserialize, Debug)]
pub struct TxAuthorization {
    pub authorization_code: Option<String>,
    pub bin: Option<String>,
    pub last4: Option<String>,
    pub exp_month: Option<String>,
    pub exp_year: Option<String>,
    pub channel: Option<String>,
    pub card_type: Option<String>,
    pub bank: Option<String>,
    pub country_code: Option<String>,
    pub brand: Option<String>,
    pub reusable: Option<bool>,
    pub signature: Option<String>,
    pub account_name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TxCustomer {
    pub id: u32,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub customer_code: Option<String>,
    pub phone: Option<String>,
    pub metadata: Option<String>,
    pub risk_action: Option<String>,
    pub international_format_phone: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub next: Option<String>,
    pub previous: Option<String>,
    pub per_page: Option<i64>,
}

#[derive(Deserialize, Debug)]
pub struct TransactionsResponse {
    pub status: bool,
    pub message: Option<String>,
    pub data: Vec<TxData>,
    pub meta: Meta,
}

#[derive(Deserialize, Debug)]
pub struct Timeline {
    pub status: bool,
    pub message: String,
    pub data: TimeLineData,
}

#[derive(Deserialize, Debug)]
pub struct TimeLineData {
    pub start_time: Option<i64>,
    pub time_spent: Option<i32>,
    pub attempts: Option<i32>,
    pub errors: Option<i32>,
    pub success: bool,
    pub mobile: bool,
    pub input: Vec<Value>,
    pub history: Vec<HistoryData>,
}

#[derive(Deserialize, Debug)]
pub struct HistoryData {
    pub r#type: String,
    pub message: String,
    pub time: i32,
}

#[derive(Deserialize, Debug)]
pub struct ExportTransaction {
    pub status: bool,
    pub message: String,
    pub data: ExportData,
}

#[derive(Deserialize, Debug)]
pub struct ExportData {
    pub path: String,
    pub expires_at: String,
}

// InitializeTransaction initializes a new transaction
pub async fn initialize_transaction(client: &PaystackClient, req: InitializeTransactionRequest,
) -> Result<InitializeTransactionResponse, io::Error> {
    const PATH: &str = "transaction/initialize";
    let body = match serde_json::to_string(&req) {
        Ok(body) => body,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Serialization failed: {}", e))),
    };
    let response = client.make_request(Method::POST, PATH, Some(body)).await?;
    let status_code = response.status().as_u16();
    let body = match response.text().await {
        Ok(body) => body,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to read response: {}", e))),
    };
    if status_code != 200 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Paystack API error: status {}, body: {}", status_code, body),
        ));
    }
    let tx_resp = match serde_json::from_str::<InitializeTransactionResponse>(&body) {
        Ok(resp) => resp,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Deserialization failed: {} (body: {})", e, body))),
    };
    if !tx_resp.status {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Transaction initialization failed: {}", tx_resp.message),
        ));
    }
    Ok(tx_resp)
}

// VerifyTransaction verifies a transaction
pub async fn verify_transaction(client: &PaystackClient, req: VerifyTransactionRequest,
) -> Result<TransactionResponse, io::Error> {
    let path = format!("transaction/verify/{}", req.reference.trim_start_matches('/'));
    let response = client.make_request(Method::GET, &path, None).await?;
    let status_code = response.status().as_u16();
    let body = match response.text().await {
        Ok(body) => body,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to read response: {}", e))),
    };
    if status_code != 200 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Paystack API error: status {}, body: {}", status_code, body),
        ));
    }
    let verify_tx = match serde_json::from_str::<TransactionResponse>(&body) {
        Ok(resp) => resp,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Deserialization failed: {} (body: {})", e, body))),
    };
    if !verify_tx.status {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Transaction verification failed: {}", verify_tx.message),
        ));
    }
    Ok(verify_tx)
}

// FetchTransaction fetches a transaction
pub async fn fetch_transaction(client: &PaystackClient, txid: &str,
) -> Result<TransactionResponse, io::Error> {
    let path = format!("transaction/{}", txid.trim_start_matches('/'));
    let response = client.make_request(Method::GET, &path, None).await?;
    let status_code = response.status().as_u16();
    let body = match response.text().await {
        Ok(body) => body,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to read response: {}", e))),
    };
    if status_code != 200 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Paystack API error: status {}, body: {}", status_code, body),
        ));
    }
    let fetch_tx = match serde_json::from_str::<TransactionResponse>(&body) {
        Ok(resp) => resp,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Deserialization failed: {} (body: {})", e, body))),
    };
    if !fetch_tx.status {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Fetch transaction failed: {}", fetch_tx.message),
        ));
    }
    Ok(fetch_tx)
}

// ChargeAuthorization charges a card using an authorization code
pub async fn charge_authorization(client: &PaystackClient, req: ChargeAuthorizationRequest,
) -> Result<ChargeAuthorizationResponse, io::Error> {
    const PATH: &str = "transaction/charge_authorization";
    let body = match serde_json::to_string(&req) {
        Ok(body) => body,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Serialization failed: {}", e))),
    };
    let response = client.make_request(Method::POST, PATH, Some(body)).await?;
    let status_code = response.status().as_u16();
    let body = match response.text().await {
        Ok(body) => body,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to read response: {}", e))),
    };
    if status_code != 200 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Paystack API error: status {}, body: {}", status_code, body),
        ));
    }
    let charge_tx = match serde_json::from_str::<ChargeAuthorizationResponse>(&body) {
        Ok(resp) => resp,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Deserialization failed: {} (body: {})", e, body))),
    };
    if !charge_tx.status {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Charge authorization failed: {}", charge_tx.message),
        ));
    }
    Ok(charge_tx)
}

// FetchAllTransaction fetches all transactions created on the account
pub async fn fetch_all_transaction(client: &PaystackClient) -> Result<TransactionsResponse, io::Error> {
    const PATH: &str = "transaction";
    let response = client.make_request(Method::GET, PATH, None).await?;
    let status_code = response.status().as_u16();
    let body = match response.text().await {
        Ok(body) => body,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to read response: {}", e))),
    };
    if status_code != 200 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Paystack API error: status {}, body: {}", status_code, body),
        ));
    }
    let all_txs = match serde_json::from_str::<TransactionsResponse>(&body) {
        Ok(resp) => resp,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Deserialization failed: {} (body: {})", e, body))),
    };
    if !all_txs.status {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Fetch all transactions failed: {}", all_txs.message.unwrap_or("No message provided".to_string())),
        ));
    }
    Ok(all_txs)
}

// TransactionTimeline fetches the timeline of a transaction
pub async fn transaction_timeline(client: &PaystackClient, txid: &str) -> Result<Timeline, io::Error> {
    let path = format!("transaction/timeline/{}", txid.trim_start_matches('/'));
    let response = client.make_request(Method::GET, &path, None).await?;
    let status_code = response.status().as_u16();
    let body = match response.text().await {
        Ok(body) => body,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to read response: {}", e))),
    };
    if status_code != 200 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Paystack API error: status {}, body: {}", status_code, body),
        ));
    }
    let timeline = match serde_json::from_str::<Timeline>(&body) {
        Ok(resp) => resp,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Deserialization failed: {} (body: {})", e, body))),
    };
    if !timeline.status {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Transaction timeline failed: {}", timeline.message),
        ));
    }
    Ok(timeline)
}
