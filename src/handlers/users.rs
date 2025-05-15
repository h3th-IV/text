use actix_web::{web, HttpResponse, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash
};
use sqlx::MySqlPool;

use crate::models::users::{CreateUser, LoginUser, User, UserCart};

use super::users_carts::RawUserCart;
use crate::models::cart::Cart;


pub async fn create_user(
    pool: web::Data<MySqlPool>,
    user: web::Json<CreateUser>,
) -> impl Responder {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let new_password = argon2.hash_password(user.password.as_bytes(), &salt).unwrap().to_string();
    let users = sqlx::query_as!(
        User,
        "insert into users(name, email, password, balance, role) values (?,?,?,?,?)",
        user.name,
        user.email,
        new_password,
        0,
        "student"
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
        let f_user_val = match fetch_user {
            Some(ref uc) => &uc.user,
            None => return HttpResponse::BadRequest().json("Invalid credentials passed"),
        };
        
        let parsed_hash = PasswordHash::new(&f_user_val.password).unwrap();
        println!("{} \n\n {} ", f_user_val.password, parsed_hash);
        if f_user_val.password == parsed_hash.to_string(){
            HttpResponse::Ok().json(fetch_user)
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


pub async fn fetch_user(pool: web::Data<MySqlPool>, email:String) -> Option<UserCart> {
    let raw_result = sqlx::query_as::<_, RawUserCart>(
        r#"
        SELECT 
            u.id as user_id, 
            u.name as user_name, 
            u.email as user_email, 
            u.password as user_password, 
            u.balance as user_balance, 
            u.total_profit as user_total_profit, 
            u.total_losses as user_total_losses, 
            u.is_admin as user_is_admin, 
            u.is_approved as user_is_approved, 
            u.is_blocked as user_is_blocked, 
            u.grof_points as user_grof_points, 
            u.role as user_role,
            c.id as cart_id, 
            c.role as cart_role, 
            c.email as cart_email, 
            c.total_order_amount as cart_total_order_amount, 
            c.created_at as cart_created_at, 
            c.updated_at as cart_updated_at
        from users u
        left join cart c on u.email = c.email
        where u.email = ?
        "#,
    )
    .bind(        email
    )
    .fetch_one(pool.get_ref())
    .await;

    match raw_result {
        Ok(raw) => Some(UserCart {
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
            cart: raw.cart_id.map(|id| Cart {
                id,
                role: raw.cart_role.unwrap_or_default(),
                email: raw.cart_email.unwrap_or_default(),
                total_order_amount: raw.cart_total_order_amount.unwrap_or(0),
                created_at: raw.cart_created_at,
                updated_at: raw.cart_updated_at,
            }),
        }),
        Err(e) => {
            eprintln!("Error fetching user: {}", e);
            None
        }
    }
}

pub async fn fetch_user_with_cart(pool: web::Data::<MySqlPool>, email: web::Path<String>) -> impl Responder{
    let email_: String = email.clone();
    let user_cart = fetch_user(pool, email.into_inner()).await;
    match user_cart {
        Some(uc) => HttpResponse::Ok().json(uc),
        None => {
            let message = format!("User with email '{}' not found", &email_);
            HttpResponse::NotFound().body(message)
        }
    }
}