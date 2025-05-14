use actix_web::{web, HttpResponse, Responder};
use sqlx::MySqlPool;

use crate::models::cart::{Cart, CreateCart};

pub async fn create_cart(pool: web::Data<MySqlPool>, cart: web::Json<CreateCart>) -> impl Responder {
    let create_cart = sqlx::query_as!(
        Cart,
        "insert into cart (role, email, total_order_amount) values (?,?,?)",
        cart.role,
        cart.email,
        cart.total_order_amount
    )
    .execute(pool.get_ref())
    .await;
    match create_cart {
        Ok(cart) => {
            let cart_id = cart.last_insert_id() as i64;
            let cart_details = sqlx::query_as!(
                Cart,
                "select id, role, email, total_order_amount, created_at, updated_at from cart where id = ?",
                cart_id
            ).fetch_one(pool.get_ref()).await;
            match cart_details {
                Ok(cart_result) => {
                    println!("{:?}", cart);
                    HttpResponse::Ok().json(cart_result)
                },
                Err(e) => {
                    eprintln!("{}", e);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            HttpResponse::BadRequest().finish()
        }
    }
}
