use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Transaction{
    pub id: i64,
    pub cart_id: i64,
    pub email: String,
    pub access_code: String,
    pub reference: String,
    pub amount: i64, // in kobo
    pub status: String, // pending, success, failed
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct TxnResponse{
    pub id: i64,
    pub cart_id: i64,
    pub email: String,
    pub access_code: String,
    pub reference: String,
    pub amount: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}