use actix_web::{web, HttpResponse, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::models::{
    cart::Cart, user_cart::CartUser, users::{CreateUser, LoginUser, User}
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
                HttpResponse::Ok().json(ruser)
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

pub async fn login_user(
    pool: web::Data<MySqlPool>,
    user: web::Json<LoginUser>,
) -> impl Responder {
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
        HttpResponse::Ok().json(user_data)
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

#[derive(Debug,Serialize, Deserialize)]
pub struct EmailQ {
    email:String
}

pub async fn fetch_user(
    pool: web::Data<MySqlPool>,
    email:String,
) -> Result<CartUser, sqlx::Error> {
    let single_user = sqlx::query_as::<_, User>("select * from users where email = ?")
        .bind(email)
        .fetch_one(pool.get_ref())
        .await;
    if let Err(sqlx::Error::RowNotFound) = single_user {
        HttpResponse::NotFound().json("user not found");
        return Ok(CartUser::new());
    }

    let su = single_user.unwrap();
    let single_user_cart = sqlx::query_as::<_, CartUser>(
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
    c.updated_at AS cart_updated_at
        FROM users u
        LEFT JOIN cart c ON u.email = c.email
        "#
    )
    .bind(su.email.clone())
    .fetch_one(pool.get_ref())
    .await?;
    let user_email_in_cart = sqlx::query_as::<_,Cart>("select * from cart where email = ?").bind(su.email.clone()).fetch_one(pool.get_ref()).await;
    if let Err(ce) = user_email_in_cart {
        println!("{}", ce);
        HttpResponse::NotFound().json("user does not have cart items");
        return Ok(CartUser::new());
    }
    let ueic = user_email_in_cart.unwrap();
    if ueic.email.is_empty() {
        println!("{}", "user does not exist in cart");
        return Ok(CartUser::new());
    }
    Ok(single_user_cart)
}

pub async fn fetch_single_user(pool:web::Data<MySqlPool>, query: web::Query<EmailQ>) -> impl Responder {
    let email = query.email.clone();
    println!("email returned : {}", email.clone());
    let user = fetch_user(pool, email).await.unwrap();
    if user.id < 1 || user.email.is_empty() || user.cart.email.is_none() {
        println!("{}", "invalid user returned");
        return HttpResponse::BadRequest().json("invalid user returned");
    }
     HttpResponse::Ok().json(user)
}