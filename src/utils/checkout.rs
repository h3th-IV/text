use std::fs::{self, File};
use std::io::{self, Write};
use sqlx::MySqlPool;
use time::{OffsetDateTime};

use crate::handlers::users_carts::RawUserCart;

/*
    After ordr check-out is completed by user, they can pay and
 */

pub async fn save_user_checkout_to_file(pool: &MySqlPool, email: &str) -> Result<(), io::Error> {
    let raw_result = match sqlx::query_as::<_, RawUserCart>(
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
        "#
    )
    .bind(email)
    .fetch_one(pool)
    .await {
        Ok(result) => result,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e))),
    };

    if raw_result.cart_id.is_none() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "User's cart does not exist"));
    }

    //sanitize email; 
    let sanitized_email = email.replace(['@', '.'], "_");

    //get currnt date
    let now = OffsetDateTime::now_utc();
    // let format = format_description::parse("[year repr:last_two]-[month]-[day]")?;
    // let date_str = now.format(&format).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    

    //generate filenam
    let filename = format!("checkouts/checkout_{}_{}.txt", sanitized_email, now.date().to_string());

    //create checkouts dir
    match fs::create_dir_all("checkouts") {
        Ok(()) => (),
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to create directory: {}", e))),
    };

    let mut file = File::create(&filename)?;
    writeln!(file, "Email: {}", raw_result.user_email)?;
    writeln!(file, "Name: {}", raw_result.user_name)?;
    writeln!(file, "Role: {}", raw_result.user_role)?;
    writeln!(file, "Total Order Amount: {}", raw_result.cart_total_order_amount.unwrap_or(0))?;
    writeln!(file, "Cart Created At: {:?}", raw_result.cart_created_at.unwrap().date())?;
    writeln!(file, "Cart Updated At: {:?}", raw_result.cart_updated_at.unwrap().date())?;
    file.flush()?;

    //TODO: email fiole to user
    Ok(())
}