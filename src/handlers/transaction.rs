use actix_web::{web, HttpResponse, Responder};
use sqlx::MySqlPool;
use serde::Deserialize;
use crate::{
    handlers::{cart::checkout_cart, users::fetch_user},
    models::{cart::Cart, data::{PaginatedResponse, Pagination}, transaction::{Transaction, TxnResponse}},
    paysterk::{client::{self, PaystackClient}, transaction::{initialize_transaction, verify_transaction, InitializeTransactionRequest, VerifyTransactionRequest}}, utils::timefmt::conver_off_set_date_to_date,
};

/*  
    this will trigger a new transaction for the user cart
    a checkout url will be returned by paystack, which user can use for payment
    the transaction is logged and will be updated via the webhook url with the tx
    refernce
 */
pub async fn init_transaction(pool: web::Data<MySqlPool>, cart_id: web::Path<i64>) -> impl Responder {
    let cart_id = *cart_id;

    //fetch cart
    let cart_result = sqlx::query_as::<_, Cart>(
        "SELECT id, paid, package, email, total_order_amount, created_at, updated_at FROM cart WHERE id = ?"
    )
    .bind(cart_id)
    .fetch_one(pool.get_ref())
    .await;

    let cart = match cart_result {
        Ok(c) if !c.paid => c,
        Ok(_) => return HttpResponse::BadRequest().body("Cart already paid"),
        Err(_) => return HttpResponse::NotFound().body("Cart not found"),
    };

    let user_result = fetch_user(pool.clone(), cart.email.clone()).await;
    let user = match user_result {
        Ok(u) => u,
        Err(_) => return HttpResponse::BadRequest().body("User not found"),
    };
    if user.address.is_empty() {
        return HttpResponse::BadRequest().body("User address required");
    }

    let paystack_client = match PaystackClient::new() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to create Paystack client: {}", e);
            return HttpResponse::InternalServerError().body("Failed to initialize payment");
        }
    };

    let init_req = InitializeTransactionRequest {
        email: cart.email.clone(),
        amount: cart.total_order_amount as u32,
        callback_url: Some("".to_string()), //will setup a call back ulr 
    };

    match initialize_transaction(&paystack_client, init_req).await {
        Ok(resp) => {
            if resp.status {
                let data = match resp.data.as_object() {
                    Some(data) => data,
                    None => {
                        return HttpResponse::InternalServerError().body("Invalid Paystack response data")
                    }
                };
                let authorization_url = match data.get("authorization_url").and_then(|v| v.as_str()) {
                    Some(url) => url,
                    None => {
                        return HttpResponse::InternalServerError().body("Missing authorization_url")
                    }
                };
                let access_code = match data.get("access_code").and_then(|v| v.as_str()) {
                    Some(code) => code,
                    None => {
                        return HttpResponse::InternalServerError().body("Missing access_code")
                    }
                };
                let reference = match data.get("reference").and_then(|v| v.as_str()) {
                    Some(ref_) => ref_,
                    None => {
                        return HttpResponse::InternalServerError().body("Missing reference")
                    }
                };

                //save txn
                let insert_result = sqlx::query!(
                    "INSERT INTO transactions (email, access_code, amount, status, reference, cart_id) VALUES (?, ?, ?, 'pending', ?, ?)",
                    cart.email,
                    access_code,
                    cart.total_order_amount,
                    reference,
                    cart.id
                )
                .execute(pool.get_ref())
                .await;

                if let Err(e) = insert_result {
                    eprintln!("Insert transaction error: {:?}", e);
                    return HttpResponse::InternalServerError().body("Failed to save transaction");
                }

                HttpResponse::Ok().json(serde_json::json!({
                    "authorization_url": authorization_url,
                    "access_code": access_code,
                    "reference": reference
                }))
            } else {
                eprintln!("Paystack error: {}", resp.message);
                HttpResponse::BadRequest().body(resp.message)
            }
        }
        Err(e) => {
            eprintln!("Paystack transaction error: {}", e);
            HttpResponse::InternalServerError().body("Failed to initialize transaction")
        }
    }
}

pub async fn get_all_transactions(pool: web::Data<MySqlPool>, query: web::Query<Pagination>) -> impl Responder {
    let page = query.page.max(1);
    let per_page = query.per_page.clamp(1, 100);
    let offset = (page - 1) * per_page;

    //count total txns
    let total_result = sqlx::query!("SELECT COUNT(*) as count FROM transactions")
        .fetch_one(pool.get_ref())
        .await;

    let total = match total_result {
        Ok(r) => r.count,
        Err(e) => {
            eprintln!("Count transactions error: {}", e);
            return HttpResponse::InternalServerError().body("Failed to count transactions");
        }
    };

    //paginate txns
    let transactions = sqlx::query_as::<_, Transaction>(
        "SELECT id, cart_id, email, access_code, reference, amount, status, created_at, updated_at 
         FROM transactions ORDER BY created_at DESC LIMIT ? OFFSET ?"
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;

    match transactions {
        Ok(transactions) => {
            let response = PaginatedResponse {
                data: transactions.into_iter().map(|t| TxnResponse {
                    id: t.id,
                    cart_id: t.cart_id,
                    email: t.email,
                    access_code: t.access_code,
                    reference: t.reference,
                    amount: t.amount,
                    status: t.status,
                    created_at: conver_off_set_date_to_date(t.created_at),
                    updated_at: conver_off_set_date_to_date(t.updated_at)
                }).collect(),
                page,
                per_page,
                total,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Fetch transactions error: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch transactions")
        }
    }
}

pub async fn get_transaction_by_reference(pool: web::Data<MySqlPool>, reference: web::Path<String>) -> impl Responder {
    let reference = reference.into_inner();

    let transaction = sqlx::query_as::<_, Transaction>(
        "SELECT id, cart_id, email, access_code, reference, amount, status, created_at, updated_at 
         FROM transactions WHERE reference = ?"
    )
    .bind(&reference)
    .fetch_one(pool.get_ref())
    .await;

    match transaction {
        Ok(t) => {
            let txn = TxnResponse{
                id: t.id,
                cart_id: t.cart_id,
                email: t.email,
                access_code: t.access_code,
                reference: t.reference,
                amount: t.amount,
                status: t.status,
                created_at: conver_off_set_date_to_date(t.created_at),
                updated_at: conver_off_set_date_to_date(t.updated_at)
            };
            HttpResponse::Ok().json(txn)
        },
        Err(_) => HttpResponse::NotFound().body("Transaction not found"),
    }
}


pub async  fn paystack_callback(pool: web::Data<MySqlPool>, reference: web::Path<String>) -> impl Responder{
    println!("Threader !!!!!!!!!");
    let reference = reference.into_inner();

    let paystack_client = match PaystackClient::new() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to create Paystack client: {}", e);
            return HttpResponse::InternalServerError().body("Failed to initialize client");
        }
    };

    let verify_req = VerifyTransactionRequest{
        reference: reference.clone(),
    };

    match verify_transaction(&paystack_client, verify_req).await {
        Ok(resp) if resp.status && resp.data.status.as_deref() == Some("success") => {
            match checkout_cart(pool.get_ref(), &reference).await {
                Ok(order_json) => {
                    println!("Checkout successful for reference {}: {}", reference, order_json);
                    HttpResponse::Found()
                        .append_header(("Location", "/payment/success"))
                        .body("Payment successful, order created")
                }
                Err(e) => {
                    eprintln!("Checkout error for reference {}: {}", reference, e);
                    HttpResponse::InternalServerError().body("Failed to process order")
                }
            }
        }
        Ok(resp) => {
            eprintln!("Payment verification failed: {}", resp.message);
            HttpResponse::BadRequest().body("Payment verification failed")
        }
        Err(e) => {
            eprintln!("Paystack verification error: {}", e);
            HttpResponse::InternalServerError().body("Failed to verify payment")
        }
    }
}

pub async fn payment_success() -> impl Responder {
    HttpResponse::Ok().json("Payment was successful! Your order has been placed.")
}