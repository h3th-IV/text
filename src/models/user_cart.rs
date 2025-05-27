use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

// use super::cart::Cart;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UCartResponse {
    pub id: Option<i64>,
    pub paid: Option<bool>,
    pub package: Option<String>,
    pub email: Option<String>,
    pub total_order_amount: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Default, FromRow)]
pub struct CartUser {
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
    pub grof_points: Option<i32>,
    pub role: String,
    pub phone_number: String,
    pub address: String,
    pub created_at: Option<OffsetDateTime>,
    pub all_orders: Option<sqlx::types::Json<Vec<String>>>,
    pub pending_orders: Option<sqlx::types::Json<Vec<String>>>,
    pub fufilled_orders: Option<sqlx::types::Json<Vec<String>>>,
    pub cart_id: Option<i64>,
    pub cart_paid: Option<bool>,
    pub cart_package: Option<String>,
    pub cart_email: Option<String>,
    pub cart_total_order_amount: Option<i64>,
    pub cart_created_at: Option<OffsetDateTime>,
    pub cart_updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CartUserResponse {
    pub id: i64,
    pub name: String,
    pub password: String,
    pub email: String,
    pub balance: Option<i32>,
    pub total_profit: Option<i32>,
    pub total_losses: Option<i32>,
    pub is_admin: bool,
    pub is_approved: bool,
    pub is_blocked: bool,
    pub grof_points: Option<i32>,
    pub role: String,
    pub phone_number: String,
    pub address: String,
    pub created_at: String,
    pub all_orders: Option<sqlx::types::Json<Vec<String>>>,
    pub pending_orders: Option<sqlx::types::Json<Vec<String>>>,
    pub fufilled_orders: Option<sqlx::types::Json<Vec<String>>>,
    pub cart: Vec<UCartResponse>,
}

impl CartUserResponse {
    pub fn new() -> CartUserResponse {
        CartUserResponse::default()
    }
}

impl CartUser {
    pub fn _new() -> Self {
        CartUser::default()
    }
}