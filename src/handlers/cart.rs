use actix_web::{web, HttpResponse, Responder};
use sqlx::MySqlPool;
use time::{Duration, OffsetDateTime};

use crate::{
    handlers::users::fetch_user, models::cart::{Cart, CartResponse, CreateCart, Order, UpdateCart}, paysterk::{client::PaystackClient, transaction::{InitializeTransactionRequest, initialize_transaction}}, utils::timefmt::human_readable_time,
};

/*  
    Create cart, create a cart for the user with the selected package
 */
pub async fn create_cart(pool: web::Data<MySqlPool>, cart: web::Json<CreateCart>) -> impl Responder {
    //validate package
    if !["family", "student"].contains(&cart.package.as_str()) {
        return HttpResponse::BadRequest().body("Invalid package: must be 'family' or 'student'");
    }

    let expected_amount = match cart.package.as_str() {
        "family" => 1_500_000,
        "student" => 1_000_000,
        _ => return HttpResponse::BadRequest().finish(),
    };
    if cart.total_order_amount != expected_amount {
        return HttpResponse::BadRequest().body(format!(
            "Invalid total_order_amount: expected {} kobo for {}",
            expected_amount, cart.package
        ));
    }

    let insert_result = sqlx::query!(
        "INSERT INTO cart (paid, package, email, total_order_amount) VALUES (0, ?, ?, ?)",
        cart.package,
        cart.email,
        cart.total_order_amount
    )
    .execute(pool.get_ref())
    .await;

    if insert_result.is_err() {
        eprintln!("Insert cart error: {:?}", insert_result.err());
        return HttpResponse::BadRequest().finish();
    }

    let cart_result = sqlx::query_as::<_, Cart>(
        "SELECT id, paid, package, email, total_order_amount, created_at, updated_at FROM cart WHERE email = ? ORDER BY created_at DESC LIMIT 1"
    )
    .bind(&cart.email)
    .fetch_one(pool.get_ref())
    .await;

    match cart_result {
        Ok(cart) => {
            let response = CartResponse {
                id: cart.id,
                paid: cart.paid,
                package: cart.package,
                email: cart.email,
                total_order_amount: cart.total_order_amount,
                created_at: human_readable_time(cart.created_at),
                updated_at: human_readable_time(cart.updated_at),
            };
            match serde_json::to_string(&response) {
                Ok(cart_json) => HttpResponse::Ok().body(cart_json),
                Err(e) => {
                    eprintln!("Serialization error: {}", e);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(e) => {
            eprintln!("Fetch cart error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

/*
    the checkout cart function, will be triggered upon 
    successfull payment confirmation by the webhook handler
    It will create a new order from the confirmed cart, set the 
    delivery date; the delivery date is based on my own...
 */
pub async fn _checkout_cart(pool: &MySqlPool, cart_id: i64) -> Result<String, String> {
    // Fetch cart
    let cart_result = sqlx::query_as::<_, Cart>(
        "SELECT id, paid, package, email, total_order_amount, created_at, updated_at FROM cart WHERE id = ?"
    )
    .bind(cart_id)
    .fetch_one(pool)
    .await;

    let cart = match cart_result {
        Ok(c) if !c.paid => c,
        Ok(_) => return Err("Cart already paid".to_string()),
        Err(_) => return Err("Cart not found".to_string()),
    };

    // Fetch user for address
    let user_result = fetch_user(web::Data::new(pool.clone()), cart.email.clone()).await;
    let user = match user_result {
        Ok(u) => u,
        Err(_) => return Err("User not found".to_string()),
    };
    let address = user.address;
    if address.is_empty() {
        return Err("User address required".to_string());
    }

    // Update cart to paid
    let update_result = sqlx::query!(
        "UPDATE cart SET paid = 1, updated_at = NOW() WHERE id = ?",
        cart_id
    )
    .execute(pool)
    .await;

    if let Err(e) = update_result {
        eprintln!("Update cart error: {:?}", e);
        return Err("Failed to update cart".to_string());
    }

    // Calculate delivery date
    let delivery_days = match cart.package.as_str() {
        "family" => 7,
        "student" => 3,
        _ => return Err("Invalid package".to_string()),
    };
    let delivery_date = OffsetDateTime::now_utc() + Duration::days(delivery_days);

    // Create order
    let order_result = sqlx::query!(
        "INSERT INTO orders (cart_id, status, email, address, delivery_date, created_at, updated_at) 
         VALUES (?, 'confirmed', ?, ?, ?, NOW(), NOW())",
        cart_id,
        cart.email,
        address,
        delivery_date
    )
    .execute(pool)
    .await;

    if let Err(e) = order_result {
        eprintln!("Insert order error: {:?}", e);
        return Err("Failed to create order".to_string());
    }

    // Fetch created order
    let order = sqlx::query_as::<_, Order>(
        "SELECT id, cart_id, status, email, address, delivery_date, created_at, updated_at 
         FROM orders WHERE cart_id = ?"
    )
    .bind(cart_id)
    .fetch_one(pool)
    .await;

    match order {
        Ok(o) => Ok(serde_json::to_string(&o).unwrap_or_default()),
        Err(e) => {
            eprintln!("Fetch order error: {}", e);
            Err("Failed to fetch order".to_string())
        }
    }
}

pub async fn _update_cart(
    pool: web::Data<MySqlPool>,
    path: web::Path<i64>,
    update: web::Json<UpdateCart>,
) -> impl Responder {
    let cart_id = path.into_inner();

    // Validate package
    if !["family", "student"].contains(&update.package.as_str()) {
        return HttpResponse::BadRequest().body("Invalid package: must be 'family' or 'student'");
    }

    //update package and total_order_amount
    let new_amount = match update.package.as_str() {
        "family" => 1_500_000, // ₦15000
        "student" => 1_000_000, // ₦10000
        _ => return HttpResponse::BadRequest().finish(),
    };

    let result = sqlx::query!(
        "UPDATE cart SET package = ?, total_order_amount = ?, updated_at = NOW() WHERE id = ? AND paid = 0",
        update.package,
        new_amount,
        cart_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            let cart = sqlx::query_as::<_, Cart>(
                "SELECT id, paid, package, email, total_order_amount, created_at, updated_at FROM cart WHERE id = ?"
            )
            .bind(cart_id)
            .fetch_one(pool.get_ref())
            .await;

            match cart {
                Ok(c) => HttpResponse::Ok().json(c),
                Err(_) => HttpResponse::NotFound().body("Cart not found"),
            }
        }
        Ok(_) => HttpResponse::BadRequest().body("Cart already paid or not found"),
        Err(e) => {
            eprintln!("Update cart error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn _delete_cart(pool: web::Data<MySqlPool>, path: web::Path<i64>) -> impl Responder {
    let cart_id = path.into_inner();

    let result = sqlx::query!("DELETE FROM cart WHERE id = ? AND paid = 0", cart_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => HttpResponse::Ok().body("Cart deleted"),
        Ok(_) => HttpResponse::BadRequest().body("Cart already paid or not found"),
        Err(e) => {
            eprintln!("Delete cart error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_cart(pool: web::Data<MySqlPool>, path: web::Path<i64>) -> impl Responder {
    let cart_id = path.into_inner();

    let cart = sqlx::query_as::<_, Cart>(
        "SELECT id, paid, package, email, total_order_amount, created_at, updated_at FROM cart WHERE id = ?"
    )
    .bind(cart_id)
    .fetch_one(pool.get_ref())
    .await;

    match cart {
        Ok(c) => {
            let response = CartResponse {
                id: c.id,
                paid: c.paid,
                package: c.package,
                email: c.email,
                total_order_amount: c.total_order_amount,
                created_at: human_readable_time(c.created_at),
                updated_at: human_readable_time(c.updated_at),
            };
            HttpResponse::Ok().json(response)
        }
        Err(_) => HttpResponse::NotFound().body("Cart not found"),
    }
}


/*  
    this will trigger a new 
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
                    "INSERT INTO transactions (email, access_code, amount, status) VALUES (?, ?, ?, 'pending')",
                    cart.email,
                    access_code,
                    cart.total_order_amount
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