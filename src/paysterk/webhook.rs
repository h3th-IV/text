use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use hmac::{Hmac, Mac};
use sha2::Sha512;
use hex;
use std::env;
use std::io;

//webhookEvent reps the structure of paystack event payloads
#[derive(Deserialize, Debug)]
pub struct WebhookEvent {
    pub event: String,
    #[serde(default)]
    pub data: Value, //generic data type
}

// TransactionEventData represents the data for a transaction event (e.g., charge.success)
#[derive(Deserialize, Debug)]
pub struct TransactionEventData {
    pub event: String,
    pub data: Transaction,
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

// Transaction represents the transaction data in a charge.success event
#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub id: i64,
    pub amount: i64,
    pub reference: String,
    pub status: Option<String>,
    pub currency: Option<String>,
    pub paid_at: Option<String>,
    pub created_at: Option<String>,
    pub channel: Option<String>,
    pub gateway_response: Option<String>,
    pub ip_address: Option<String>,
    #[serde(default)]
    pub metadata: Option<Value>,
    pub fees: Option<i64>,
    #[serde(default)]
    pub customer: Option<TxCustomer>,
    #[serde(default)]
    pub authorization: Option<Value>,
    #[serde(default)]
    pub log: Option<TxLog>,
}

//handle_paystack_events processes incoming Paystack webhook requests
pub async fn handle_paystack_events(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    // let secret_key = match env::var("PAYSTACK_SECRET_KEY") {
    //     Ok(key) => key,
    //     Err(_) => {
    //         eprintln!("PAYSTACK_SECRET_KEY not set");
    //         return HttpResponse::InternalServerError().body("Server configuration error");
    //     }
    // };

    let secret_key = "sk_test_fa16b06664111cf77ebcd2df5d58a1110ca0dfa6";

    //get signature from header
    let signature = match req.headers().get("x-paystack-signature") {
        Some(header) => header.to_str().unwrap_or(""),
        None => {
            eprintln!("Missing x-paystack-signature header");
            return HttpResponse::BadRequest().body("Missing signature header");
        }
    };

    //verify signature
    if !is_valid_signature(&body, signature, &secret_key) {
        eprintln!("Invalid webhook signature");
        return HttpResponse::BadRequest().body("Invalid signature");
    }

    //parse json body into WehookEvent
    let event: WebhookEvent = match serde_json::from_slice(&body) {
        Ok(event) => event,
        Err(e) => {
            eprintln!("Invalid JSON format: {}", e);
            return HttpResponse::BadRequest().body("Invalid JSON format");
        }
    };

    //process webhook based on event type
    match event.event.as_str() {
        "charge.success" => {
            //parse into TransactionEventData
            let tx_event: TransactionEventData = match serde_json::from_slice(&body) {
                Ok(tx) => tx,
                Err(e) => {
                    eprintln!("Failed to parse charge.success data: {}", e);
                    return HttpResponse::BadRequest().body("Failed to parse transaction data");
                }
            };
            if let Some(customer) = tx_event.data.customer {
                if let Some(email) = customer.email {
                    println!("Payment successful: {} (₦{})", email, tx_event.data.amount);
                } else {
                    println!("Payment successful: (no email) (₦{})", tx_event.data.amount);
                }
            } else {
                println!("Payment successful: (no customer) (₦{})", tx_event.data.amount);
            }
        }
        //other event handling come later
        _ => {
            println!("Unhandled event type: {}", event.event);
        }
    }

    // Respond with 200 OK to acknowledge receipt
    HttpResponse::Ok().body("Webhook received")
}

// is_valid_signature ensures only request from pastack is processed
fn is_valid_signature(request_body: &[u8], signature: &str, secret_key: &str) -> bool {
    if signature.is_empty() {
        return false;
    }

    //create HMAC-SHA512 hash usig the secret key'
    let mut mac = Hmac::<Sha512>::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(request_body);
    let result = mac.finalize();
    let expected_signature = hex::encode(result.into_bytes());

    //compare signatur
    signature == expected_signature
}