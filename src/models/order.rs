use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Order {
    pub id: i64,
    pub cart_id: i64, //references cart.id
    pub status: String, //"confirmed", "shipped", "delivered", "cancelled"
    pub email: String,
    pub address: String, //user delivery address
    pub delivery_date: OffsetDateTime,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct OrderResponse {
    pub id: i64,
    pub cart_id: i64, //references cart.id
    pub status: String, //"confirmed", "shipped", "delivered", ""cancelled""
    pub email: String,
    pub address: String, //user delivery address
    pub delivery_date: String,
    pub created_at: String,
    pub updated_at: String,
}