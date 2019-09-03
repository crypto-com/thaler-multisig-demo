use actix_web::{web, Error as AWError};
use diesel;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use failure::Error;
use futures::Future;

use crate::models::{Order, OrderStatus};

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub fn execute_is_order_exist(
    pool: web::Data<Pool>,
    order_id: String,
) -> impl Future<Item = bool, Error = AWError> {
    web::block(move || is_order_exist(pool, order_id)).from_err()
}
pub fn execute_register_order(
    pool: web::Data<Pool>,
    order: Order,
) -> impl Future<Item = bool, Error = AWError> {
    web::block(move || register_order(pool, order)).from_err()
}
pub fn execute_store_payment_transaction_id(
    pool: web::Data<Pool>,
    order_id: String,
    payment_transaction_id: String,
) -> impl Future<Item = bool, Error = AWError> {
    web::block(move || store_payment_transaction_id(pool, order_id, payment_transaction_id))
        .from_err()
}
pub fn execute_get_order_by_id(
    pool: web::Data<Pool>,
    order_id: String,
) -> impl Future<Item = Order, Error = AWError> {
    web::block(move || get_order_by_id(pool, order_id)).from_err()
}
pub fn execute_update_order_status(
    pool: web::Data<Pool>,
    order_id: String,
    status: OrderStatus,
) -> impl Future<Item = bool, Error = AWError> {
    web::block(move || update_order_status(pool, order_id, status)).from_err()
}
pub fn execute_store_exchanged_data(
    pool: web::Data<Pool>,
    order_id: String,
    session_id: String,
    settlement_transaction_id: String,
) -> impl Future<Item = bool, Error = AWError> {
    web::block(move || store_submit_data(pool, order_id, session_id, settlement_transaction_id))
        .from_err()
}
pub fn execute_get_orders_by_status(
    pool: web::Data<Pool>,
    status_list: Vec<OrderStatus>,
) -> impl Future<Item = Vec<Order>, Error = AWError> {
    web::block(move || get_orders_by_status(pool, status_list)).from_err()
}

fn is_order_exist(pool: web::Data<Pool>, id: String) -> Result<bool, Error> {
    use crate::schema::orders::dsl::*;
    let conn: &SqliteConnection = &pool.get().unwrap();

    let result = orders.filter(order_id.eq(&id)).first::<Order>(conn);
    match result {
        Ok(_) => Ok(true),
        Err(err) => match err {
            diesel::result::Error::NotFound => Ok(false),
            _ => Err(err.into()),
        },
    }
}

fn register_order(pool: web::Data<Pool>, order: Order) -> Result<bool, Error> {
    use crate::schema::orders;
    let conn: &SqliteConnection = &pool.get().unwrap();

    diesel::insert_into(orders::table)
        .values(&order)
        .execute(conn)
        .expect("Error saving new post");
    Ok(true)
}

fn store_payment_transaction_id(
    pool: web::Data<Pool>,
    affected_order_id: String,
    transaction_id: String,
) -> Result<bool, Error> {
    use crate::schema::orders::dsl::*;
    let conn: &SqliteConnection = &pool.get().unwrap();
    diesel::update(orders.filter(order_id.eq(&affected_order_id)))
        .set((
            payment_transaction_id.eq(&transaction_id),
            status.eq(OrderStatus::PendingResponse),
        ))
        .execute(conn)
        .expect("store_payment_transaction_id error");
    Ok(true)
}

fn store_submit_data(
    pool: web::Data<Pool>,
    affected_order_id: String,
    new_session_id: String,
    new_settlement_transaction_id: String,
) -> Result<bool, Error> {
    use crate::schema::orders::dsl::*;
    let conn: &SqliteConnection = &pool.get().unwrap();
    diesel::update(orders.filter(order_id.eq(&affected_order_id)))
        .set((
            session_id.eq(&new_session_id),
            settlement_transaction_id.eq(&new_settlement_transaction_id),
        ))
        .execute(conn)
        .expect("store_submit_data error");
    Ok(true)
}

fn get_order_by_id(pool: web::Data<Pool>, id: String) -> Result<Order, Error> {
    use crate::schema::orders::dsl::*;
    let conn: &SqliteConnection = &pool.get().unwrap();
    let result = orders
        .filter(order_id.eq(&id))
        .first::<Order>(conn)
        .expect("get_order_by_id error");
    Ok(result)
}

fn update_order_status(
    pool: web::Data<Pool>,
    affected_order_id: String,
    new_status: OrderStatus,
) -> Result<bool, Error> {
    use crate::schema::orders::dsl::*;
    let conn: &SqliteConnection = &pool.get().unwrap();
    diesel::update(orders.filter(order_id.eq(&affected_order_id)))
        .set(status.eq(new_status))
        .execute(conn)
        .expect("update_order_status error");
    Ok(true)
}

fn get_orders_by_status(
    pool: web::Data<Pool>,
    order_status: Vec<OrderStatus>,
) -> Result<Vec<Order>, Error> {
    use crate::schema::orders::dsl::*;
    let conn: &SqliteConnection = &pool.get().unwrap();
    let result = orders
        .filter(status.eq_any(order_status))
        .load::<Order>(conn)
        .expect("Error loading orders");
    Ok(result)
}
