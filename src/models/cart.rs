use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

/* Shopping cart and order models for managing user purchases with role-specific packages.
   - Cart:
     - Tracks pre-payment state with package (family, student) and payment status (paid or not).
     - Uses i64 for ID, total order amount for Paystack.
     - Unpaid carts expire after 24 hours.
   - Order:
     - Created after cart payment is confirmed (via Paystack webhook).
     - Tracks post-payment state with status (confirmed, shipped, delivered), email, address, and delivery date.
     - References cart via cart_id.
   - Integrated with Paystack for payment processing.
*/

// pub enum Status{
//     #[sqlx(rename = "pending")]
//     Pending,
//     #[sqlx(rename = "paid")]
//     Paid,
//     #[sqlx(rename = "delivered")]
//     Delivered,
//     #[sqlx(rename = "returned")]
//     Returned
// }

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Cart {
    pub id: i64,
    pub paid: bool,
    pub package: String,
    pub email: String,
    pub total_order_amount: i64,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct CreateCart {
    pub package: String,
    pub email: String,
    pub total_order_amount: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct UpdateCart {
    pub package: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Order {
    pub id: i64,
    pub cart_id: i64, //references cart.id
    pub status: String, //"confirmed", "shipped", "delivered"
    pub email: String,
    pub address: String, //user delivery address
    pub delivery_date: Option<OffsetDateTime>,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}