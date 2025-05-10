use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize,Deserialize, Clone, FromRow)]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub quantity: i32
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct CreateItem {
    pub name: String,
    pub quantity: i32
}
