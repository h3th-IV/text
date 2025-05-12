use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub name: String,
    pub password: String,
    pub email: String,
    pub balance: i32
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CreateUser {
    pub name: String,
    pub password: String,
    pub email: String
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct LoginUser {
    pub email: String,
    pub password:String
}