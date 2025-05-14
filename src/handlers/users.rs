use actix_web::{web, HttpResponse, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash
};
use sqlx::MySqlPool;

use crate::{
    models::users::{CreateUser, LoginUser, User},
};

pub async fn create_user(
    pool: web::Data<MySqlPool>,
    user: web::Json<CreateUser>,
) -> impl Responder {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let new_password = argon2.hash_password(user.password.as_bytes(), &salt).unwrap().to_string();
    let users = sqlx::query_as!(
        User,
        "insert into users(name, email, password, balance) values (?,?,?,?)",
        user.name,
        user.email,
        new_password,
        0
    )
    .execute(pool.get_ref())
    .await;

    match users {
        Ok(user) => {
            let id = user.last_insert_id();
            let ret_user = sqlx::query_as!(
                User,
                "select id, name, email, password, balance,is_admin, is_approved,grof_points,total_profit, total_losses, is_blocked, role from users where id = ?",
                id
            )
            .fetch_one(pool.get_ref())
            .await;
            match ret_user {
                Ok(ruser) => HttpResponse::Ok().json(ruser),
                Err(e) => {
                    eprintln!("{}", e);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn fetch_user(pool: web::Data<MySqlPool>, email:String) -> Option<User> {
    let user = sqlx::query_as!(User, "select id, name, email, password, balance,is_admin, is_approved,grof_points,total_profit, total_losses, is_blocked, role from users where email = ?",email)
        .fetch_one(pool.get_ref())
        .await;
    match user {
        Ok(u) => {
            return Some(u);
        }
        Err(e) => {
            eprintln!("{}", e);
            HttpResponse::BadRequest().finish();
            return None;
        }
    }
}

pub async fn login_user(pool: web::Data<MySqlPool>, user: web::Json<LoginUser>) -> impl Responder {
    if user.email.is_empty() {
        return HttpResponse::Ok().json("email is empty");
    }

    if user.password.is_empty() {
        return HttpResponse::Ok().json("password is empty");
    }

    if user.password.len() < 8 {
        return HttpResponse::BadRequest().json("invalid password");
    }

    let user_exists = sqlx::query_as!(
        User,
        "select id, name, email, password, balance,is_admin, is_approved,grof_points,total_profit, total_losses, is_blocked, role from users where email = ?",
        user.email
    )
    .fetch_one(pool.get_ref())
    .await;
    if let Ok(user_e) = user_exists {
        let fetch_user = fetch_user(pool,user_e.email.clone()).await;
        let f_user_val = fetch_user.unwrap();
        
        let parsed_hash = PasswordHash::new(&f_user_val.password).unwrap();
        if f_user_val.password == parsed_hash.to_string(){
            HttpResponse::Ok().json(f_user_val)
        } else {
            HttpResponse::BadRequest().json("Invalid credentials passed")
        }
    } else {
        HttpResponse::BadRequest().json("invalid credentials passed")
    }
}

pub async fn update_user_balance(
    pool: web::Data<MySqlPool>,
    user_id: i64,
    new_balance: i32,
) -> impl Responder {
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
            let new_balance = sqlx::query_as!(User, "select id, name, email, password, balance,is_admin, is_approved,grof_points,total_profit, total_losses, is_blocked, role from users where id = ?", updated_user).fetch_one(pool.get_ref()).await;
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
