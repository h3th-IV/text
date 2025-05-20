use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

use super::cart::Cart;


#[derive(Debug, Serialize, Deserialize, Clone, Default,FromRow)]
pub struct UCart {
    #[sqlx(rename = "cart_id")]
    pub id: Option<i64>,
    #[sqlx(rename = "cart_role")]
    pub role: Option<String>,
    #[sqlx(rename = "cart_email")]
    pub email: Option<String>,
    #[sqlx(rename = "cart_total_order_amount")]
    pub total_order_amount: Option<i64>,
    #[sqlx(rename = "cart_created_at")]
    pub created_at: Option<OffsetDateTime>,
    #[sqlx(rename = "cart_updated_at")]
    pub updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize,Default, FromRow)]
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
    #[sqlx(flatten)]
    pub cart: UCart,
}

impl CartUser {
    pub fn new() -> Self {
        CartUser::default()
    }
}