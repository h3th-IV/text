use actix_web::{web, HttpResponse, Responder};
use sqlx::MySqlPool;
use crate::models::users::{User, UserCart};
use crate::models::cart::Cart;

//temp struct with all fields from user and cart
#[derive(sqlx::FromRow)]
struct RawUserCart {
    //user fields
    user_id: i64,
    user_name: String,
    user_email: String,
    user_password: String,
    user_balance: Option<i32>,
    user_total_profit: Option<i32>,
    user_total_losses: Option<i32>,
    user_is_admin: Option<i8>,
    user_is_approved: Option<i8>,
    user_is_blocked: Option<i8>,
    user_grof_points: Option<i32>,
    user_role: String,
    //cart fields (nullable dur to left joim)
    cart_id: Option<i64>,
    cart_role: Option<String>,
    cart_email: Option<String>,
    cart_total_order_amount: Option<i64>,
    cart_created_at: Option<time::OffsetDateTime>,
    cart_updated_at: Option<time::OffsetDateTime>,
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
                    password: raw.user_password,
                    balance: raw.user_balance,
                    total_profit: raw.user_total_profit,
                    total_losses: raw.user_total_losses,
                    is_admin: raw.user_is_admin,
                    is_approved: raw.user_is_approved,
                    is_blocked: raw.user_is_blocked,
                    grof_points: raw.user_grof_points,
                    role: raw.user_role,
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