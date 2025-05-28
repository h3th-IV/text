use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Transaction{
    pub id: i64,
    pub email: String,
    pub access_code: String,
    pub amount: i64, // in kobo
    pub status: String, // pending, success, failed
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
