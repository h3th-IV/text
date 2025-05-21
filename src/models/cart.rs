use arrayvec::ArrayVec;
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

    /* we also want to have a fixed delivery date based on when we */
    /* can actually deliver the product. this would help in flexibility */
    pub products: Option<sqlx::types::Json<Vec<String>>>,

    /* this model would be updated after payment via paystack */
    /* we want the cart to be the main lookup model for the user */
    /* that's the reason it's embedded inside the fetch user call */
    pub cart_paid: Option<bool>,
    pub cart_paid_amount: Option<i64>,
    pub cart_paid_date: Option<OffsetDateTime>, 
    pub cart_delivery_date: Option<OffsetDateTime>,

    /* user can change items in the cart for a certain number of times */
    /* and after a certain number we lock modifications and delete after */
    /* 6 hours if `cart_paid` is still false or if `cart_paid_amount` is */
    /* lesser than the total_order_amount */
    pub cart_modified: Option<i32>
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct CreateCart {
    pub role: String,
    pub email: String,
    pub products: Option<sqlx::types::Json<Vec<String>>>,
    pub total_order_amount: String
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct UpdateCart {
    pub role: String,
}