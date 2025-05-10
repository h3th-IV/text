mod handlers;
mod models;

use dotenvy::dotenv;
use handlers::{create_items,get_items};
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
    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(pool.clone()))
        .route("/create-item", web::post().to(create_items))
        .route("/", web::get().to(get_items))
    }).bind(("127.0.0.1", 8080))?.run().await
}
