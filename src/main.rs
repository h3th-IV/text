mod handlers;
mod models;
mod utils;
mod paysterk;

use dotenvy::dotenv;
use handlers::{
    cart::{create_cart, get_all_carts, get_cart, get_user_carts, update_cart}, 
    items::{create_items, get_items}, 
    order::{cancel_order, get_all_orders, get_order_by_id, get_user_orders, mark_delivered, mark_order_shipped}, 
    transaction::{get_all_transactions, get_transaction_by_reference, init_transaction, paystack_callback, payment_success}, 
    users::{create_user, fetch_single_user, login_user}
};
use paysterk::{client, webhook::handle_paystack_events, transaction};
use std::{env, io};

use actix_web::{web, App, HttpServer};
use sqlx::mysql::MySqlPoolOptions;

#[tokio::main]
async fn main() -> io::Result<()> {
    let paysterk_client = client::PaystackClient::new()?;
    let init_req = transaction::InitializeTransactionRequest{
        email: "samuelbonux10@gmail.com".to_string(),
        amount: 10000,
        callback_url: Some("".to_string()),
    };
    match transaction::initialize_transaction(&paysterk_client, init_req).await {
        Ok(resp) => println!("{:#?}", resp),
        Err(e) => println!("err creating tx: {:#?}", e)
    }

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
            .route("/cart/{id}", web::get().to(get_cart))
            .route("/update-cart/{id}", web::put().to(update_cart))
            // .route("/delete-cart/{id}", web::delete().to(delete_cart))
            .route("/user", web::get().to(fetch_single_user))
            .route("/webhook", web::post().to(handle_paystack_events))
            .route("/init-txn/{id}", web::post().to(init_transaction))
            .route("/{id}/shipped", web::put().to(mark_order_shipped))
            .route("/{id}/delivered", web::put().to(mark_delivered))
            .route("/carts", web::get().to(get_all_carts))
            .route("/orders", web::get().to(get_all_orders))
            .route("/user/{email}/cart", web::get().to(get_user_carts))
            .route("/{id}/order", web::get().to(get_order_by_id))
            .route("/user/{email}/order", web::get().to(get_user_orders))
            .route("/{id}/cancel/order", web::post().to(cancel_order))
            .route("/txns", web::get().to(get_all_transactions))
            .route("/txn/{reference}", web::get().to(get_transaction_by_reference))
            .route("/paystack-callback/{reference}", web::get().to(paystack_callback))
            .route("/payment/success", web::get().to(payment_success))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}