use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

/* for the add to cart function we're going to have some fields */
/*
    - role (family or student)
    - email (user's email for tracking)
    - total_order_amount (based on role package)
    - created_at (check time of order)
    - updated_at (was the order updated ?)

    for the update field user's should be able to change their roles
    and if done, the total_order_amount would change automatically
    - role (family or student)

    when the proceed to checkout button is clicked that is where we
    get to activate the registration for user and update the cart 
    information and pass them to stripe or paystack for payments
*/

#[derive(Debug, Serialize, Deserialize, Clone,FromRow)]
pub struct Cart {
    pub id : i64,
    pub role: String,
    pub email: String,
    pub total_order_amount: i64,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct CreateCart {
    pub role: String,
    pub email: String,
    pub total_order_amount: String
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct UpdateCart {
    pub role: String,
}