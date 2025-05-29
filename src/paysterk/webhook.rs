use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::Value;
use hmac::{Hmac, Mac};
use sha2::Sha512;
use hex;
use sqlx::MySqlPool;

use crate::handlers::cart::_checkout_cart;

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
    pub start_time: Option<u32>,
    pub time_spent: Option<u32>,
    pub attempts: Option<u32>,
    pub errors: Option<u32>,
    pub success: Option<bool>,
    pub mobile: Option<bool>,
    pub input: Option<Vec<String>>,
    pub tx_hist: Option<Vec<TxLogHistory>>,
}

#[derive(Deserialize, Debug)]
pub struct TxLogHistory {
    pub r#type: Option<String>,
    pub message: Option<String>,
    pub time: Option<u32>,
}


#[derive(Deserialize, Debug)]
pub struct TxCustomer {
    pub id: Option<u32>,
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
    //"sk_test_fa16b06664111cf77ebcd2df5d58a1110ca0dfa6"
}

//handle_paystack_events processes incoming Paystack webhook requests
pub async fn handle_paystack_events(req: HttpRequest, body: web::Bytes, pool: web::Data<MySqlPool>) -> impl Responder {
    // let secret_key = match env::var("PAYSTACK_SECRET_KEY") {
    //     Ok(key) => key,
    //     Err(_) => {
    //         eprintln!("PAYSTACK_SECRET_KEY not set");
    //         return HttpResponse::Ok().body("Webhook received");
    //     }
    // };

    let secret_key = "sk_test_fa16b06664111cf77ebcd2df5d58a1110ca0dfa6"; //neglect this and also neglect the comment above it

    //get signature from header
    // let signature = match req.headers().get("x-paystack-signature") {
    //     Some(header) => header.to_str().unwrap_or_default(),
    //     None => {
    //         eprintln!("Missing x-paystack-signature header");
    //         return HttpResponse::Ok().body("Webhook received");
    //     }
    // };

    //verify signature
    // if !is_valid_signature(&body, signature, &secret_key) {
    //     eprintln!("Invalid webhook signature");
    //     return HttpResponse::Ok().body("Webhook received");
    // }

    //parse json body into WehookEvent
    let event: WebhookEvent = match serde_json::from_slice(&body) {
        Ok(event) => event,
        Err(e) => {
            eprintln!("Invalid JSON format: {}", e);
            return HttpResponse::Ok().body("Webhook received");
        }
    };

    println!("{:#?}", event);

    //we'll only process only charge.success
    if event.event == "charge.success" {

        /*
            async func here will init the checkout function as paystack requires immediate response on charge.success
        */
        actix_web::rt::spawn(async move {
            //parse datainto TransactionEventData
            let tx_event: TransactionEventData = match serde_json::from_slice(&body) {
                Ok(tx) => tx,
                Err(e) => {
                    eprintln!("Failed to parse charge.success data: {}", e);
                    return;
                }
            };

            //verify status
            if tx_event.data.status.as_deref() != Some("success") {
                eprintln!("Charge success event with non-success status: {:?}", tx_event.data.status);
                return;
            }

            //checkout cart
            match _checkout_cart(pool.get_ref(), &tx_event.data.reference).await {
                Ok(order_json) => {
                    println!("Checkout successful for reference {}: {}", tx_event.data.reference, order_json);
                }
                Err(e) => {
                    eprintln!("Checkout error for reference {}: {}", tx_event.data.reference, e);
                }
            }
        });
    } else {
        println!("Ignored event: {}", event.event);
    }

    //immediately return 200 OK
    HttpResponse::Ok().body("Webhook received")
}