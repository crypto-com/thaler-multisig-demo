use actix_web::{web, Error as AWError};
use failure::Error;
use futures::Future;
use diesel;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use crate::NewOrderRequest;
use crate::OrderDetails;
use crate::OrderStatus;

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub fn execute_register_order(
    pool: web::Data<Pool>,
    params: web::Query<NewOrderRequest>,
) -> impl Future<Item = bool, Error = AWError> {
    web::block(move || register_order(pool, params)).from_err()
}
pub fn execute_store_payment_transaction_id(
    pool: web::Data<Pool>,
    order_id: String,
    payment_transaction_id: String,
) -> impl Future<Item = bool, Error = AWError> {
    web::block(move || store_payment_transaction_id(pool, order_id, payment_transaction_id)).from_err()
}
pub fn execute_get_order_by_id(
    pool: web::Data<Pool>,
    order_id: String,
) -> impl Future<Item = OrderDetails, Error = AWError> {
    web::block(move || get_order_by_id(pool, order_id)).from_err()
}
pub fn execute_mark_order_status(
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
    web::block(move || store_submit_data(pool, order_id, session_id, settlement_transaction_id)).from_err()
}
pub fn execute_get_orders_by_status(
    pool: web::Data<Pool>,
    status_list: Vec<OrderStatus>,
) -> impl Future<Item = Vec<OrderDetails>, Error = AWError> {
    web::block(move || get_orders_by_status(pool, status_list)).from_err()
}
fn register_order(pool: web::Data<Pool>, params: web::Query<NewOrderRequest>) -> Result<bool, Error> {
    use backend::schema::order_details;
    let conn: &SqliteConnection = &pool.get().unwrap();
    let order_details = OrderDetails {
        order_id: params.order_id.to_string(),
        amount: params.amount.to_string(),
        status: OrderStatus::PendingPayment,
        buyer_public_key: params.buyer_public_key.to_string(),
        buyer_view_key: params.buyer_view_key.to_string(),
        buyer_address: params.buyer_address.to_string(),
        escrow_public_key: params.escrow_public_key.to_string(),
        escrow_view_key: params.escrow_view_key.to_string(),
        session_id: "".to_string(),
        payment_transaction_id: "".to_string(),
        settlement_transaction_id: "".to_string(),
    };
    diesel::insert_into(order_details::table)
        .values(&order_details)
        .execute(conn)
        .expect("Error saving new post");
    Ok(true)
}

fn store_payment_transaction_id(
    pool: web::Data<Pool>,
    affected_order_id: String,
    transaction_id: String,
) -> Result<bool, Error> {
    use backend::schema::order_details::dsl::*;
    let conn: &SqliteConnection = &pool.get().unwrap();
    diesel::update(order_details.filter(order_id.eq(&affected_order_id)))
        .set(payment_transaction_id.eq(&transaction_id))
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
    use backend::schema::order_details::dsl::*;
    let conn: &SqliteConnection = &pool.get().unwrap();
    diesel::update(order_details.filter(order_id.eq(&affected_order_id)))
        .set((session_id.eq(&new_session_id), settlement_transaction_id.eq(&new_settlement_transaction_id)))
        .execute(conn)
        .expect("store_submit_data error");
    Ok(true)
}

fn get_order_by_id(pool: web::Data<Pool>, id: String) -> Result<OrderDetails, Error> {
    use backend::schema::order_details::dsl::*;
    let conn: &SqliteConnection = &pool.get().unwrap();
    let result = order_details
        .filter(order_id.eq(&id))
        .first::<OrderDetails>(conn)
        .expect("get_order_by_id error");
    Ok(result)
}

fn update_order_status(pool: web::Data<Pool>, affected_order_id: String, new_status: OrderStatus) -> Result<bool, Error> {
    use backend::schema::order_details::dsl::*;
    let conn: &SqliteConnection = &pool.get().unwrap();
    diesel::update(order_details.filter(order_id.eq(&affected_order_id)))
        .set(status.eq(new_status))
        .execute(conn)
        .expect("update_order_status error");
    Ok(true)
}

fn get_orders_by_status(pool: web::Data<Pool>, order_status: Vec<OrderStatus>) -> Result<Vec<OrderDetails>, Error> {
    use backend::schema::order_details::dsl::*;
    let conn: &SqliteConnection = &pool.get().unwrap();
    let result = order_details
        .filter(status.eq_any(order_status))
        .load::<OrderDetails>(conn)
        .expect("Error loading orders");
    Ok(result)
}
