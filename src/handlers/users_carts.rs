use actix_web::{web, HttpResponse, Responder};
use sqlx::MySqlPool;
use crate::models::users::{User, UserCart};
use crate::models::cart::Cart;
use crate::utils::checkout::save_user_checkout_to_file;

//temp struct with all fields from user and cart
#[derive(sqlx::FromRow)]
pub struct RawUserCart {
    //user fields
    pub user_id: i64,
    pub user_name: String,
    pub user_email: String,
    pub user_password: String,
    pub user_balance: Option<i32>,
    pub user_total_profit: Option<i32>,
    pub user_total_losses: Option<i32>,
    pub user_is_admin: Option<i8>,
    pub user_is_approved: Option<i8>,
    pub user_is_blocked: Option<i8>,
    pub user_grof_points: Option<i32>,
    pub user_role: String,
    pub user_phone_number: String,
    pub user_address: String,
    //cart fields (nullable dur to left joim)
    pub cart_id: Option<i64>,
    pub cart_role: Option<String>,
    pub cart_email: Option<String>,
    pub cart_total_order_amount: Option<i64>,
    pub cart_created_at: Option<time::OffsetDateTime>,
    pub cart_updated_at: Option<time::OffsetDateTime>,
}

pub async fn fetch_users_carts(pool: web::Data<MySqlPool>) -> impl Responder {
    let raw_results = sqlx::query_as::<_, RawUserCart>(
        r#"
        SELECT 
            u.id AS user_id, 
            u.name AS user_name, 
            u.email AS user_email, 
            u.password AS user_password, 
            u.balance AS user_balance, 
            u.total_profit AS user_total_profit, 
            u.total_losses AS user_total_losses, 
            u.is_admin AS user_is_admin, 
            u.is_approved AS user_is_approved, 
            u.is_blocked AS user_is_blocked, 
            u.grof_points AS user_grof_points, 
            u.role AS user_role,
            u.phone_number as user_phone_number,
            u.address as user_address,
            c.id AS cart_id, 
            c.role AS cart_role, 
            c.email AS cart_email, 
            c.total_order_amount AS cart_total_order_amount, 
            c.created_at AS cart_created_at, 
            c.updated_at AS cart_updated_at
        FROM users u
        LEFT JOIN cart c ON u.email = c.email
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    match raw_results {
        Ok(raw_carts) => {
            let user_carts: Vec<UserCart> = raw_carts.into_iter().map(|raw| UserCart {
                user: User {
                    id: raw.user_id,
                    name: raw.user_name,
                    email: raw.user_email,
                    password: "".to_string(),
                    balance: raw.user_balance,
                    total_profit: raw.user_total_profit,
                    total_losses: raw.user_total_losses,
                    is_admin: raw.user_is_admin,
                    is_approved: raw.user_is_approved,
                    is_blocked: raw.user_is_blocked,
                    grof_points: raw.user_grof_points,
                    role: raw.user_role,
                    phone_number: raw.user_phone_number,
                    address: raw.user_address
                },
                cart: match raw.cart_id {
                    Some(id) => Some(Cart {
                        id,
                        role: raw.cart_role.unwrap_or_default(),
                        email: raw.cart_email.unwrap_or_default(),
                        total_order_amount: raw.cart_total_order_amount.unwrap_or(0),
                        created_at: raw.cart_created_at,
                        updated_at: raw.cart_updated_at,
                    }),
                    None => None,
                },
            }).collect();
            HttpResponse::Ok().json(user_carts)
        }
        Err(e) => {
            eprintln!("Error fetching users and carts: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

//use raw strings for query later

pub async fn save_checkout(
    pool: web::Data<MySqlPool>,
    email: web::Path<String>,
) -> impl Responder {
    let result = save_user_checkout_to_file(pool.get_ref(), &email).await;
    match result {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"message": "Checkout saved"})),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()})),
    }
}