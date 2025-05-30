use actix_web::{web, HttpResponse, Responder};
use sqlx::MySqlPool;

use crate::{
models::{data::*, order::*}, utils::timefmt::{human_readable_time, conver_off_set_date_to_date},
};

pub async fn mark_order_shipped(
    pool: web::Data<MySqlPool>,
    order_id: web::Path<i64>,
) -> impl Responder {
    let order_id = order_id.into_inner();

    let result = sqlx::query!(
        "UPDATE orders SET status = 'shipped', updated_at = NOW() WHERE id = ? AND status = 'confirmed'",
        order_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            let order = sqlx::query_as::<_, Order>(
                "SELECT id, cart_id, status, email, address, delivery_date, created_at, updated_at FROM orders WHERE id = ?"
            )
            .bind(order_id)
            .fetch_one(pool.get_ref())
            .await;

            match order {
                Ok(o) => {
                    let response = OrderResponse {
                        id: o.id,
                        cart_id: o.cart_id,
                        status: o.status,
                        email: o.email,
                        address: o.address,
                        delivery_date: human_readable_time(o.delivery_date),
                        created_at: human_readable_time(o.created_at),
                        updated_at: human_readable_time(o.updated_at),
                    };
                    HttpResponse::Ok().json(response)
                },
                Err(_) => HttpResponse::NotFound().body("Order not found"),
            }
        }
        Ok(_) => HttpResponse::BadRequest().body("Order not found or not in confirmed status"),
        Err(e) => {
            eprintln!("Update order error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn mark_delivered(
    pool: web::Data<MySqlPool>,
    order_id: web::Path<i64>,
) -> impl Responder {
    let order_id = order_id.into_inner();

    let result = sqlx::query!(
        "UPDATE orders SET status = 'delivered', updated_at = NOW() WHERE id = ? AND status = 'shipped'",
        order_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            let order = sqlx::query_as::<_, Order>(
                "SELECT id, cart_id, status, email, address, delivery_date, created_at, updated_at FROM orders WHERE id = ?"
            )
            .bind(order_id)
            .fetch_one(pool.get_ref())
            .await;

            match order {
                Ok(o) => {
                    let response = OrderResponse {
                        id: o.id,
                        cart_id: o.cart_id,
                        status: o.status,
                        email: o.email,
                        address: o.address,
                        delivery_date: human_readable_time(o.delivery_date),
                        created_at: human_readable_time(o.created_at),
                        updated_at: human_readable_time(o.updated_at),
                    };
                    HttpResponse::Ok().json(response)
                },
                Err(_) => HttpResponse::NotFound().body("Order not found"),
            }
        }
        Ok(_) => HttpResponse::BadRequest().body("Order not found or not in shipped status"),
        Err(e) => {
            eprintln!("Update order error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}


pub async fn get_all_orders(
    pool: web::Data<MySqlPool>,
    query: web::Query<Pagination>,
) -> impl Responder {
    let page = query.page.max(1);
    let per_page = query.per_page.clamp(1, 100);
    let offset = (page - 1) * per_page;

    //count total orders
    let total_result = sqlx::query!("SELECT COUNT(*) as count FROM orders")
        .fetch_one(pool.get_ref())
        .await;

    let total = match total_result {
        Ok(r) => r.count,
        Err(e) => {
            eprintln!("Count orders error: {}", e);
            return HttpResponse::InternalServerError().body("Failed to count orders");
        }
    };

    //fetch paginate orders
    let orders = sqlx::query_as::<_, Order>(
        "SELECT id, cart_id, status, email, address, delivery_date, created_at, updated_at 
         FROM orders ORDER BY created_at DESC LIMIT ? OFFSET ?"
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;

    match orders {
        Ok(orders) => {
            let response = PaginatedResponse {
                data: orders.into_iter().map(|o| OrderResponse {
                    id: o.id,
                    cart_id: o.cart_id,
                    status: o.status,
                    email: o.email,
                    address: o.address,
                    delivery_date: conver_off_set_date_to_date(o.delivery_date),
                    created_at: human_readable_time(o.created_at),
                    updated_at: human_readable_time(o.updated_at),
                }).collect(),
                page,
                per_page,
                total,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Fetch orders error: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch orders")
        }
    }
}
