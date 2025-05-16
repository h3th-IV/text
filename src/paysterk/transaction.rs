use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
pub struct InitializeTransactionRequest {
    pub email: String,
    pub amount: u32,
}

#[derive(Deserialize)]
pub struct InitializeTransactionResponse {
    pub status: bool,
    pub message: String,
    pub data: Value,
}

#[derive(Serialize)]
pub struct VerifyTransactionRequest {
    pub reference: String,
}

#[derive(Deserialize)]
pub struct TransactionResponse {
    pub status: bool,
    pub message: String,
    pub data: TxData,
}

#[derive(Deserialize)]
pub struct TxData {
    pub id: i32,
    pub domain: String,
    pub status: String,
    pub reference: String,
    pub receipt_number: String,
    pub amount: u32,
    pub message: String,
    pub gateway_response: String,
    pub paid_at: String,
    pub created_at: String,
    pub channel: String,
    pub currency: String,
    pub ip_address: String,
    pub metadata: Value, // Go's interface{} -> serde_json::Value
    pub tx_log: TxLog,
    pub fees: u32,
    pub fees_split: String,
    pub authorization: TxAuthorization,
    pub customer: TxCustomer,
    pub plan: Value, // map[string]interface{}
    pub split: Value, // map[string]interface{}
    pub order_id: String,
    pub requested_amount: u32,
    pub pos_transaction_data: String,
    pub source: Txsource,
    pub fees_breakdown: String,
    pub connect: String,
    pub transaction_date: String,
    pub plan_object: Value, // map[string]interface{}
    pub sub_account: Value, // map[string]interface{}
}

#[derive(Deserialize)]
pub struct Txsource {
    pub r#type: String,
    pub source: String,
    pub identifier: String,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct TxLogHistory {
    pub r#type: String,
    pub message: String,
    pub time: u32,
}

#[derive(Deserialize)]
pub struct TxAuthorization {
    pub authorization_code: String,
    pub bin: String,
    pub last4: String,
    pub exp_month: String,
    pub exp_year: String,
    pub channel: String,
    pub card_type: String,
    pub bank: String,
    pub country_code: String,
    pub brand: String,
    pub reusable: bool,
    pub signature: String,
    pub account_name: String,
}

#[derive(Deserialize)]
pub struct TxCustomer {
    pub id: u32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub customer_code: String,
    pub phone: String,
    pub metadata: String,
    pub risk_action: String,
    pub international_format_phone: String,
}

#[derive(Deserialize)]
pub struct Meta {
    pub next: String,
    pub previous: String,
    pub per_page: i32,
}

#[derive(Deserialize)]
pub struct TransactionsResponse {
    pub status: bool,
    pub message: String,
    pub data: Vec<TxData>,
    pub meta: Meta,
}

#[derive(Deserialize)]
pub struct Timeline {
    pub status: bool,
    pub message: String,
    pub data: TimeLineData,
}

#[derive(Deserialize)]
pub struct TimeLineData {
    pub start_time: i64,
    pub time_spent: i32,
    pub attempts: i32,
    pub errors: i32,
    pub success: bool,
    pub mobile: bool,
    pub input: Vec<Value>, // Go's []interface{}
    pub history: Vec<HistoryData>,
}

#[derive(Deserialize)]
pub struct HistoryData {
    pub r#type: String,
    pub message: String,
    pub time: i32,
}

#[derive(Deserialize)]
pub struct ExportTransaction {
    pub status: bool,
    pub message: String,
    pub data: ExportData,
}

#[derive(Deserialize)]
pub struct ExportData {
    pub path: String,
    pub expires_at: String,
}