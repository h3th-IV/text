use std::{ffi::CString, fs::File, os::fd::FromRawFd};

use actix_web::{web, HttpResponse, Responder};
use memmap2::MmapMut;
use serde_json::to_string;
use sqlx::MySqlPool;

use crate::{handlers::users::fetch_user, models::cart::{Cart, CreateCart}};

pub async fn create_cart(pool: web::Data<MySqlPool>, cart: web::Json<CreateCart>) -> impl Responder {
    let create_cart = sqlx::query_as!(
        Cart,
        "insert into cart (role, email, total_order_amount) values (?,?,?)",
        cart.role,
        cart.email,
        cart.total_order_amount
    )
    .execute(pool.get_ref())
    .await;
    match create_cart {
        Ok(cart) => {
            let cart_id = cart.last_insert_id() as i64;
            let cart_details = sqlx::query_as!(
                Cart,
                "select id, role, email, total_order_amount, created_at, updated_at from cart where id = ?",
                cart_id
            ).fetch_one(pool.get_ref()).await;
            match cart_details {
                Ok(cart_result) => {
                    println!("{:?}", cart);
                    /* write a functionality that would store the order or cart */
                    /* information to a temporary file that would be sent to the */
                    /* users .. (specific user making the order) */

                    let convert_data = to_string(&cart_result).unwrap();
                    println!("convert data resp : {}", convert_data);

                    let user = fetch_user(pool, cart_result.email.clone()).await.unwrap();
                    let file_path = CString::new("/tmp/".to_string() + user.name.as_str() + ".txt").unwrap();
                    let file_desc = unsafe { libc::open(file_path.as_ptr(), libc::O_RDWR | libc::O_CREAT | libc::O_TRUNC, 0o644)};
                    if file_desc < 0 { 
                        eprintln!("{}", "file to open file");
                        return HttpResponse::BadRequest().finish();
                    }
                    let user_file = unsafe { File::from_raw_fd(file_desc) };
                    user_file.set_len(convert_data.len() as u64).unwrap();

                    let mut file_process_kern = unsafe {MmapMut::map_mut(&user_file).unwrap()};
                    file_process_kern[..convert_data.len()].copy_from_slice(convert_data.as_bytes());
                    file_process_kern.flush().expect("failed to flush memory map");

                    /* 
                        when this file is created here for instance. we are going
                        to send the user an email at this point. immediately the 
                        email is sent. we delete the file. which means that the filename
                        should be the {email}.txt

                        the file should be created in memory without overheads ...
                     */
                    HttpResponse::Ok().json(cart_result)
                },
                Err(e) => {
                    eprintln!("{}", e);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            HttpResponse::BadRequest().finish()
        }
    }
}
