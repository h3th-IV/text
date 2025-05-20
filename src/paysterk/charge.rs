use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io;

use crate::paysterk::client::PaystackClient;
use reqwest::Method;

// ChargeRequest represents the request body for initiating a charge
#[derive(Serialize, Debug)]
pub struct ChargeRequest {
    pub email: String,
    pub amount: i64,
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

#[derive(Serialize, Debug)]
pub struct CardDetails {
    pub number: String,
    pub cvv: String,
    pub expiry_month: String,
    pub expiry_year: String,
}

#[derive(Serialize, Debug)]
pub struct BankDetails {
    pub code: String,
    pub account_number: String,
}

#[derive(Serialize, Debug)]
pub struct MobileMoneyDetails {
    pub phone: String,
    pub provider: String,
}

#[derive(Serialize, Debug)]
pub struct ChargeAuthorizationRequest {
    pub email: String,
    pub amount: String,
    pub authorization_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

#[derive(Serialize, Debug)]
pub struct SubmitPinRequest {
    pub reference: String,
    pub pin: String,
}

#[derive(Serialize, Debug)]
pub struct SubmitOtpRequest {
    pub reference: String,
    pub otp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<Vec<CustomFields>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank: Option<Bank>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthday: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomFields {
    pub value: Option<String>,
    pub display_name: Option<String>,
    pub variable_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bank {
    pub code: Option<String>,
    pub account_number: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ChargeResponse {
    pub status: bool,
    pub message: String,
    pub data: ChargeResponseData,
}

#[derive(Deserialize, Debug)]
pub struct ChargeResponseData {
    pub reference: String,
    pub status: Option<String>,
    pub url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ChargeAuthorizationResponse {
    pub status: bool,
    pub message: String,
    pub data: ChargeAuthData,
}

#[derive(Deserialize, Debug)]
pub struct ChargeAuthData {
    pub id: i64,
    pub amount: i64,
    pub reference: String,
    pub domain: Option<String>,
    pub status: Option<String>,
    pub receipt_number: Option<String>,
    pub message: Option<String>,
    pub gateway_response: Option<String>,
    pub paid_at: Option<String>,
    pub created_at: Option<String>,
    pub channel: Option<String>,
    pub currency: Option<String>,
    pub ip_address: Option<String>,
    #[serde(default)]
    pub metadata: Option<u32>,
    #[serde(default)]
    pub tx_log: Option<ChargeLog>,
    pub fees: Option<i64>,
    pub fees_split: Option<String>,
    #[serde(default)]
    pub authorization: Option<ChargeAuthorization>,
    #[serde(default)]
    pub customer: Option<ChargeCustomer>,
    pub plan: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ChargeLog {
    pub start_time: Option<u32>,
    pub time_spent: Option<u32>,
    pub attempts: Option<u32>,
    pub errors: Option<u32>,
    pub success: Option<bool>,
    pub mobile: Option<bool>,
    pub input: Option<Vec<String>>,
    pub tx_hist: Option<Vec<ChargeLogHistory>>,
}

#[derive(Deserialize, Debug)]
pub struct ChargeLogHistory {
    pub r#type: Option<String>,
    pub message: Option<String>,
    pub time: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChargeAuthorization {
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
pub struct ChargeCustomer {
    pub id: u32,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub customer_code: Option<String>,
    pub phone: Option<String>,
    pub metadata: Option<Metadata>,
    pub risk_action: Option<String>,
    pub international_format_phone: Option<String>,
}

// ChargeData represents the data of a charge response.
#[derive(Deserialize, Debug)]
pub struct ChargeData {
    pub id: Option<i64>,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    pub transaction_date: Option<String>,
    pub status: Option<String>,
    pub reference: Option<String>,
    pub domain: Option<String>,
    pub metadata: Option<Metadata>,
    pub gateway_response: Option<String>,
    pub message: Option<String>,
    pub channel: Option<String>,
    pub ip_address: Option<String>,
    #[serde(default)]
    pub log: Option<ChargeLog>,
    pub fees: Option<i64>,
    #[serde(default)]
    pub authorization: Option<ChargeAuthorization>,
    #[serde(default)]
    pub customer: Option<ChargeCustomer>,
    pub plan: Option<Value>,
}

pub async fn create_charge(client: &PaystackClient, req: ChargeRequest) -> Result<ChargeResponse, io::Error> {
    const PATH: &str = "charge";
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
    let charge_tx = match serde_json::from_str::<ChargeResponse>(&body) {
        Ok(resp) => resp,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Deserialization failed: {} (body: {})", e, body))),
    };
    if !charge_tx.status {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Charge failed: {}", charge_tx.message),
        ));
    }
    Ok(charge_tx)
}

pub async fn charge_authorization(client: &PaystackClient, req: ChargeAuthorizationRequest) -> Result<ChargeAuthorizationResponse, io::Error> {
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

pub async fn submit_pin(client: &PaystackClient, req: SubmitPinRequest) -> Result<ChargeAuthorizationResponse, io::Error> {
    const PATH: &str = "charge/submit_pin";
    if req.pin.len() != 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("PIN must be 4 digits, got {} digit", req.pin.len()),
        ));
    }
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
            format!("Submit PIN failed: {}", charge_tx.message),
        ));
    }
    Ok(charge_tx)
}

pub async fn submit_otp(client: &PaystackClient, req: SubmitOtpRequest) -> Result<ChargeAuthorizationResponse, io::Error> {
    const PATH: &str = "charge/submit_otp";
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
            format!("Submit OTP failed: {}", charge_tx.message),
        ));
    }
    Ok(charge_tx)
}

pub async fn check_pending_charge(client: &PaystackClient, reference: &str) -> Result<ChargeAuthorizationResponse, io::Error> {
    let path = format!("charge/{}", reference.trim_start_matches('/'));
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
    let charge_tx = match serde_json::from_str::<ChargeAuthorizationResponse>(&body) {
        Ok(resp) => resp,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Deserialization failed: {} (body: {})", e, body))),
    };
    if !charge_tx.status {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Check pending charge failed: {}", charge_tx.message),
        ));
    }
    Ok(charge_tx)
}
