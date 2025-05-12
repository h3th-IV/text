use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub password: String,
    pub email: String,
    pub balance: Option<i32>,
    pub total_profit: Option<i32>,
    pub total_losses: Option<i32>,
    pub is_admin: Option<i8>,
    pub is_approved: Option<i8>,
    pub is_blocked: Option<i8>,
    pub grof_points: Option<i32>
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UpdateBalance {
    pub id: i64,
    pub balance: Option<i32>,
}

pub struct _UpdateUserStatus {
    pub id: i64,
    pub is_admin: Option<i8>,
    pub is_approved: Option<i8>,
    pub is_blocked: Option<i8>,
    pub grof_points: Option<i32>
}