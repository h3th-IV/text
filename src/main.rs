mod handlers;
mod models;
mod utils;
mod paysterk;

use dotenvy::dotenv;
use handlers::{
    cart::create_cart,
    items::{create_items, get_items},
    users::{create_user, fetch_user_with_cart, login_user}, 
    users_carts::{fetch_users_carts, save_checkout},
};
use paysterk::{charge::{create_charge, BankDetails, ChargeAuthorizationRequest, ChargeRequest}, client::PaystackClient, transaction::{charge_authorization, fetch_all_transaction, fetch_transaction, verify_transaction, VerifyTransactionRequest}};
use std::{env, io};

use actix_web::{web, App, HttpServer};
use sqlx::mysql::MySqlPoolOptions;

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("database not set");
    let pool = MySqlPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("failed to create pool");
    println!("brex server running @:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/register", web::post().to(create_user))
            .route("/login", web::post().to(login_user))
            .route("/create-item", web::post().to(create_items))
            .route("/", web::get().to(get_items))
            .route("/add-cart", web::post().to(create_cart))
            .route("/users-carts", web::get().to(fetch_users_carts))
            .route("user/{email}", web::get().to(fetch_user_with_cart))
            .route("/checkout/{email}", web::get().to(save_checkout))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}