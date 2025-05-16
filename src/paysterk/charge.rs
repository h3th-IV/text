use serde::{Deserialize, Serialize};
use serde_json::Value;

//ChargeRequest represents the request body for initiating a charge
#[derive(Serialize)]
pub struct ChargeRequest {
    pub email: String,
    pub amount: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>, // Default: NGN
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card: Option<CardDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank: Option<BankDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile_money: Option<MobileMoneyDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<ChargeAuthorization>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Serialize)]
pub struct CardDetails {
    pub number: String,
    pub cvv: String,
    pub expiry_month: String,
    pub expiry_year: String,
}

#[derive(Serialize)]
pub struct BankDetails {
    pub code: String,
    pub account_number: String,
}

#[derive(Serialize)]
pub struct MobileMoneyDetails {
    pub phone: String,
    pub provider: String,
}

#[derive(Serialize)]
pub struct ChargeAuthorizationRequest {
    pub email: String,
    pub amount: String,
    pub authorization_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<Vec<CustomFields>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank: Option<Bank>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthday: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CustomFields {
    pub value: String,
    pub display_name: String,
    pub variable_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Bank {
    pub code: String,
    pub account_number: String,
}

#[derive(Deserialize)]
pub struct ChargeAuthorizationResponse {
    pub status: bool,
    pub message: String,
    pub data: ChargeData,
}

#[derive(Deserialize)]
pub struct ChargeAuthData {
    pub id: u32,
    pub domain: String,
    pub status: bool,
    pub reference: String,
    pub receipt_number: String,
    pub amount: u16,
    pub message: String,
    pub gateway_response: String,
    pub paid_at: String,
    pub created_at: String,
    pub channel: String,
    pub currency: String,
    pub ip_address: String,
    pub metadata: Metadata,
    pub tx_log: ChargeLog,
    pub fees: u32,
    pub fees_split: String,
    pub authorization: ChargeAuthorization,
    pub customer: ChargeCustomer,
    pub plan: String,
}

#[derive(Deserialize)]
pub struct ChargeLog {
    pub start_time: u32,
    pub time_spent: u32,
    pub attempts: u32,
    pub errors: u32,
    pub success: bool,
    pub mobile: bool,
    pub input: Vec<String>,
    pub tx_hist: Vec<ChargeLogHistory>,
}

#[derive(Deserialize)]
pub struct ChargeLogHistory {
    pub r#type: String,
    pub message: String,
    pub time: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ChargeAuthorization {
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
pub struct ChargeCustomer {
    pub id: u32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub customer_code: String,
    pub phone: String,
    pub meta_data: Metadata,
    pub risk_action: String,
    pub international_format_phone: String,
}

// ChargeData represents the data of a charge response.
#[derive(Deserialize)]
pub struct ChargeData {
    pub id: i32,
    pub amount: i32,
    pub currency: String,
    pub transaction_date: String,
    pub status: String,
    pub reference: String,
    pub domain: String,
    pub metadata: Metadata,
    pub gateway_response: String,
    pub message: String,
    pub channel: String,
    pub ip_address: String,
    pub log: ChargeLog,
    pub fees: i32,
    pub authorization: ChargeAuthorization,
    pub customer: ChargeCustomer,
    pub plan: Value,
}