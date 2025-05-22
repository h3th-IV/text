use actix_web::{web, HttpResponse, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::{
    models::{
        cart::Cart,
        user_cart::{CartUser, CartUserResponse, UCart, UCartResponse},
        users::{CreateUser, LoginUser, User, UserResponse},
    },
    utils::timefmt::human_readable_time,
};

pub async fn create_user(
    pool: web::Data<MySqlPool>,
    user: web::Json<CreateUser>,
) -> impl Responder {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let new_password = argon2
        .hash_password(user.password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    println!("Hashed password: {}", new_password);
    let users = sqlx::query_as!(
        User,
        "insert into users(name, email, password, balance, role, phone_number, address) values (?,?,?,?,?,?,?)",
        user.name,
        user.email,
        new_password,
        0,
        "student",
        user.phone_number,
        user.address
    )
    .execute(pool.get_ref())
    .await;

    match users {
        Ok(user) => {
            let id = user.last_insert_id();
            let ret_user = sqlx::query_as::<_, User>("select * from users where id = ?")
                .bind(id.clone())
                .fetch_one(pool.get_ref())
                .await;
            if let Ok(ruser) = ret_user {
                let u = fetch_user(pool, ruser.email.clone()).await.unwrap();
                let hrf = ruser.created_at.map(|d| human_readable_time(d)).unwrap();
                let res = UserResponse {
                    id: ruser.id,
                    name: ruser.name,
                    email: ruser.email,
                    balance: u.balance.unwrap_or(0),
                    total_profit: u.balance.unwrap_or(0),
                    total_losses: u.total_losses.unwrap_or(0),
                    is_admin: u.is_admin,
                    is_approved: u.is_approved,
                    is_blocked: u.is_blocked,
                    grof_points: u.grof_points.unwrap_or(0),
                    role: ruser.role,
                    phone_number: ruser.phone_number,
                    address: ruser.address,
                    created_at: hrf,
                    all_orders: u.all_orders,
                    pending_orders: u.pending_orders,
                    fufilled_orders: u.fufilled_orders,
                };
                HttpResponse::Ok().json(res)
            } else {
                let e = ret_user.err().unwrap();
                println!("{}", e);
                HttpResponse::InternalServerError().finish()
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn login_user(pool: web::Data<MySqlPool>, user: web::Json<LoginUser>) -> impl Responder {
    if user.email.trim().is_empty() {
        return HttpResponse::BadRequest().json("Email is empty");
    }

    if user.password.trim().is_empty() {
        return HttpResponse::BadRequest().json("Password is empty");
    }

    if user.password.len() < 8 {
        return HttpResponse::BadRequest().json("Invalid password length");
    }

    let result = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(user.email.clone())
        .fetch_one(pool.get_ref())
        .await;

    let user_exists = match result {
        Ok(user) => user,
        Err(_) => {
            println!("{}", "user does not exist");
            return HttpResponse::Unauthorized().json("Invalid credentials");
        }
    };

    let parsed_hash = match PasswordHash::new(&user_exists.password) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError().json("Hash parsing error");
        }
    };

    println!("Stored hash: {}", user_exists.password);
    println!("Entered password: {}", user.password);

    let is_valid = argon2::Argon2::default()
        .verify_password(user.password.as_bytes(), &parsed_hash)
        .is_ok();

    if is_valid {
        let mut user_data = user_exists;
        user_data.password.clear();
        let u = fetch_user(pool, user_data.email.clone()).await.unwrap();
        let hrf = user_data
            .created_at
            .map(|d| human_readable_time(d))
            .unwrap();
        let res = UserResponse {
            id: user_data.id,
            name: user_data.name,
            email: user_data.email,
            balance: u.balance.unwrap_or(0),
            total_profit: u.balance.unwrap_or(0),
            total_losses: u.total_losses.unwrap_or(0),
            is_admin: u.is_admin,
            is_approved: u.is_approved,
            is_blocked: u.is_blocked,
            grof_points: u.grof_points.unwrap_or(0),
            role: user_data.role,
            phone_number: user_data.phone_number,
            address: user_data.address,
            created_at: hrf,
            all_orders: u.all_orders,
            pending_orders: u.pending_orders,
            fufilled_orders: u.fufilled_orders,
        };
        HttpResponse::Ok().json(res)
    } else {
        println!("{}", "passed invalid data");
        HttpResponse::Unauthorized().json("Invalid credentials")
    }
}

pub async fn _update_user_balance(
    pool: web::Data<MySqlPool>,
    user_id: i64,
    new_balance: i32,
) -> impl Responder {
    if user_id.is_negative() || user_id == 0 {
        eprintln!("{}", "invalid user id passed");
        HttpResponse::BadRequest().finish();
    }

    let update_b = sqlx::query_as!(
        User,
        "update users set balance = ? where id = ?",
        new_balance,
        user_id
    )
    .execute(pool.get_ref())
    .await;
    match update_b {
        Ok(updated) => {
            let updated_user = updated.last_insert_id();
            let new_balance = sqlx::query_as::<_, User>("select * from users where id = ?")
                .bind(updated_user)
                .fetch_one(pool.get_ref())
                .await;
            if let Ok(new_balance_) = new_balance {
                HttpResponse::Ok().json(new_balance_)
            } else {
                HttpResponse::BadRequest().finish()
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            HttpResponse::BadRequest().finish()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailQ {
    email: String,
}

pub async fn fetch_user_old(
    pool:web::Data<MySqlPool>,
    email:String
)->Result<CartUserResponse, sqlx::Error>  {
    let user = sqlx::query_as::<_, User> (
        "select * from users where email = ?"
    ).bind(email.clone()).fetch_one(pool.get_ref()).await.unwrap();

    let carts = sqlx::query_as::<_, Cart>(
        "select * from carts where email = ?"
    ).bind(email.clone()).fetch_all(pool.get_ref()).await.unwrap();

    let human_time = user
            .created_at
            .map(|dt| human_readable_time(dt))
            .unwrap();

    let cart_res = CartUserResponse {
        id: user.id,
        name: user.name,
        email: user.email,
        password: user.password,
        balance: user.balance,
        total_profit: user.total_profit,
        total_losses: user.total_losses,
        is_admin: if user.is_admin.is_none() {
            false
        } else {
            true
        },
        is_approved: if user.is_approved.is_none() {
            false
        } else {
            true
        },
        is_blocked: if user.is_blocked.is_none() {
            false
        } else {
            true
        },
        grof_points: user.grof_points,
        role: user.role,
        phone_number: user.phone_number,
        address: user.address,
        created_at: human_time,
        all_orders: user.all_orders,
        pending_orders: user.pending_orders,
        fufilled_orders: user.fufilled_orders,
        cart: carts.into_iter().map(|m| {
            UCartResponse {
                id: Some(m.id),
                role: Some(m.role),
                email: Some(m.email),
                total_order_amount: Some(m.total_order_amount),
                created_at: "".to_string(),
                updated_at: "".to_string(),
            }
        }).collect::<Vec::<UCartResponse>>()
    };
    Ok(cart_res)
}

pub async fn fetch_user(
    pool: web::Data<MySqlPool>,
    email: String,
) -> Result<CartUserResponse, sqlx::Error> {
    let single_user = sqlx::query_as::<_, User>("select * from users where email = ?")
        .bind(email)
        .fetch_one(pool.get_ref())
        .await;
    if let Err(sqlx::Error::RowNotFound) = single_user {
        HttpResponse::NotFound().json("user not found");
        return Ok(CartUserResponse::new());
    }

    let su = single_user.unwrap();
    let single_user_carts = sqlx::query_as::<_, CartUser>(
        r#"
        SELECT 
            u.id,
    u.name,
    u.email,
    u.password,
    u.balance,
    u.total_profit,
    u.total_losses,
    u.is_admin,
    u.is_approved,
    u.is_blocked,
    u.grof_points,
    u.phone_number,
    u.role,
    u.address,
    u.created_at,
    u.all_orders,
    u.pending_orders,
    u.fufilled_orders,
    c.id AS cart_id, 
    c.role AS cart_role, 
    c.email AS cart_email, 
    c.total_order_amount AS cart_total_order_amount, 
    c.created_at AS cart_created_at, 
    c.updated_at AS cart_updated_at,
    c.products AS cart_products,
    c.cart_paid AS cart_paid,
    c.cart_paid_amount AS cart_paid_amount,
    c.cart_paid_date AS cart_paid_date,
    c.cart_delivery_date AS cart_delivery_date,
    c.cart_modified AS cart_modified
        FROM users u
        LEFT JOIN cart c ON u.email = c.email
        WHERE u.email = ?
        "#,
    )
    .bind(su.email.clone())
    .fetch_all(pool.get_ref())
    .await?;
    let user_email_in_cart = sqlx::query_as::<_, Cart>("select * from cart where email = ?")
        .bind(su.email.clone())
        .fetch_one(pool.get_ref())
        .await;
    if let Err(ce) = user_email_in_cart {
        println!("{}", ce);
        HttpResponse::NotFound().json("user does not have cart items");
        return Ok(CartUserResponse::new());
    }
    let ueic = user_email_in_cart.unwrap();
    if ueic.email.is_empty() {
        println!("{}", "user does not exist in cart");
        return Ok(CartUserResponse::new());
    }

    if single_user_carts.is_empty() {
        return Ok(CartUserResponse::new());
    }

    for single_user_cart in &single_user_carts {
        let human_time = single_user_cart
            .created_at
            .map(|dt| human_readable_time(dt))
            .unwrap();
        let c_created_at = single_user_cart
            .cart
            .created_at
            .map(|dt| human_readable_time(dt))
            .unwrap();
        let c_updated_at = single_user_cart
            .cart
            .updated_at
            .map(|dt| human_readable_time(dt))
            .unwrap();
        let cart_res = CartUserResponse {
            id: single_user_cart.id,
            name: single_user_cart.name.to_string(),
            email: single_user_cart.email.to_string(),
            password: single_user_cart.password.to_string(),
            balance: single_user_cart.balance,
            total_profit: single_user_cart.total_profit,
            total_losses: single_user_cart.total_losses,
            is_admin: if single_user_cart.is_admin.is_none() {
                false
            } else {
                true
            },
            is_approved: if single_user_cart.is_approved.is_none() {
                false
            } else {
                true
            },
            is_blocked: if single_user_cart.is_blocked.is_none() {
                false
            } else {
                true
            },
            grof_points: single_user_cart.grof_points,
            role: single_user_cart.role.clone(),
            phone_number: single_user_cart.phone_number.clone(),
            address: single_user_cart.address.clone(),
            created_at: human_time,
            all_orders: single_user_cart.all_orders.clone(),
            pending_orders: single_user_cart.pending_orders.clone(),
            fufilled_orders: single_user_cart.fufilled_orders.clone(),
            cart:single_user_carts.into_iter().map(|m| {
                UCartResponse {
                    id: m.cart.id,
                    role: m.cart.role,
                    email: m.cart.email,
                    total_order_amount: m.cart.total_order_amount,
                    created_at: c_created_at.clone(),
                    updated_at: c_updated_at.clone(),
                }
            }).collect::<Vec::<UCartResponse>>()
        };
        println!("{:?}", cart_res);
        return Ok(cart_res);
    }
    Ok(CartUserResponse::new())
}

pub async fn fetch_single_user(
    pool: web::Data<MySqlPool>,
    query: web::Query<EmailQ>,
) -> impl Responder {
    let email = query.email.clone();
    println!("email returned : {}", email.clone());
    let user = fetch_user(pool, email).await.unwrap();
    if user.id < 1 || user.email.is_empty() {
        println!("{}", "invalid user returned");
        return HttpResponse::BadRequest().json("invalid user returned");
    }
    HttpResponse::Ok().json(user)
}
