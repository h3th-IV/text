use actix_web::{web, HttpResponse, Responder};
// use serde_json::to_string;
use sqlx::MySqlPool;
use time::OffsetDateTime;

use crate::{
    handlers::users::fetch_user,
    models::cart::{Cart, CartResponse, CreateCart, Order, UpdateCart}, utils::timefmt::human_readable_time,
};

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

pub async fn _checkout_cart(pool: web::Data<MySqlPool>, path: web::Path<i64>) -> impl Responder {
    let cart_id = path.into_inner();

    // Fetch cart
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

    // Fetch user for address
    let user_result = fetch_user(pool.clone(), cart.email.clone()).await;
    let user = match user_result {
        Ok(u) => u,
        Err(_) => return HttpResponse::BadRequest().body("User not found"),
    };
    let address = user.address;
    if address.is_empty() {
        return HttpResponse::BadRequest().body("User address required");
    }

    //update cart to paid
    let update_result = sqlx::query!(
        "UPDATE cart SET paid = 1, updated_at = NOW() WHERE id = ?",
        cart_id
    )
    .execute(pool.get_ref())
    .await;

    if update_result.is_err() {
        eprintln!("Update cart error: {:?}", update_result.err());
        return HttpResponse::InternalServerError().body("Failed to update cart");
    }

    //calc delivery date
    let delivery_days = match cart.package.as_str() {
        "family" => 7,
        "student" => 3,
        _ => return HttpResponse::InternalServerError().body("Invalid package"),
    };
    let delivery_date = OffsetDateTime::now_utc() + time::Duration::days(delivery_days);

    // Create order
    let order_result = sqlx::query!(
        "INSERT INTO orders (cart_id, status, email, address, delivery_date, created_at, updated_at) 
         VALUES (?, 'confirmed', ?, ?, ?, NOW(), NOW())",
        cart_id,
        cart.email,
        address,
        delivery_date
    )
    .execute(pool.get_ref())
    .await;

    if order_result.is_err() {
        eprintln!("Insert order error: {:?}", order_result.err());
        return HttpResponse::InternalServerError().body("Failed to create order");
    }

    //fetch created order
    let order = sqlx::query_as::<_, Order>(
        "SELECT id, cart_id, status, email, address, delivery_date, created_at, updated_at 
         FROM orders WHERE cart_id = ?"
    )
    .bind(cart_id)
    .fetch_one(pool.get_ref())
    .await;

    match order {
        Ok(o) => HttpResponse::Ok().json(o),
        Err(e) => {
            eprintln!("Fetch order error: {}", e);
            HttpResponse::InternalServerError().finish()
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