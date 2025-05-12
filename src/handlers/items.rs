
use actix_web::{web, HttpResponse, Responder};
use sqlx::MySqlPool;
use crate::models::models::{Item, CreateItem};

pub async fn create_items(
    pool: web::Data<MySqlPool>,
    item: web::Json<CreateItem>
) -> impl Responder {
    let res = sqlx::query_as!(
        Item,
        "insert into items(name, quantity) values (?,?)",
        item.name, item.quantity
    ).execute(pool.get_ref()).await;

    match res {
        Ok(i) => {
            let id =  i.last_insert_id() as i64;
            let fetch_item = sqlx::query_as!(
                Item,
                "select id, name, quantity from items where id = ?",
                id
            ).fetch_one(pool.get_ref()).await;
            match fetch_item {
                Ok(f_item) => {HttpResponse::Ok().json(f_item)},
                Err(e) => {
                    eprintln!("{}",e);
                    HttpResponse::InternalServerError().finish()}
            }
        },
        Err(e) => {
            eprintln!("{}",e);
            HttpResponse::InternalServerError().finish()}
    }
}

pub async fn get_items(
    pool: web::Data<MySqlPool>
) -> impl Responder {
    let res = sqlx::query_as!(
        Item,
        "select id, name, quantity from items"
    ).fetch_one(pool.get_ref()).await;
    match res {
        Ok(f_res) => {
            HttpResponse::Ok().json(f_res)
        },
        Err(e) => {
            eprintln!("{}",e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_item(
    id: i64,
    pool: web::Data<MySqlPool>,
    item: web::Json<CreateItem>
)-> bool {
    let res = sqlx::query_as!(
        Item,
        "update items set name = ? and quantity  = ? where id = ?",
        item.name, item.quantity, id
    ).execute(pool.get_ref()).await;
    if let Ok(i_update) = res {
        let i_insert_id = i_update.last_insert_id();
        let f_update_item = sqlx::query_as!(
            Item,
            "select id, name, quantity from items where id = ?",
            i_insert_id
        ).fetch_one(pool.get_ref()).await;
        if let Ok(updated) = f_update_item {
            if updated.name == item.name 
            && updated.quantity == item.quantity {
                HttpResponse::Ok().json(updated);
                true
            } else {
                let bad_updated_name = updated.name;
                eprintln!(" updated name to {} instead of {}", bad_updated_name, item.name);
                false
            }
        } else {
            let update_err = f_update_item.err().unwrap();
            eprintln!("{}", update_err);
            false
        }
    } else {
        let err = res.err().unwrap();
        eprintln!("{}",err);
        false
    }
}