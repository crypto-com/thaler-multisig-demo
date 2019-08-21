use crate::Order;
use crate::OrderDetails;
use actix_web::{web, Error as AWError};
use failure::Error;
use futures::Future;

use diesel;
use diesel::prelude::*;

use diesel::r2d2::{self, ConnectionManager};
type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub fn execute_register_order(
    pool: web::Data<Pool>,
    params: web::Query<Order>,
) -> impl Future<Item = bool, Error = AWError> {
    web::block(move || register_order(pool, params)).from_err()
}
pub fn execute_get_order_details(
    pool: web::Data<Pool>,
    order_id: String,
) -> impl Future<Item = OrderDetails, Error = AWError> {
    web::block(move || get_order_details(pool, order_id)).from_err()
}
pub fn execute_store_session_id(
    pool: web::Data<Pool>,
    order_id: String,
    session_id: String,
) -> impl Future<Item = bool, Error = AWError> {
    web::block(move || store_session_id(pool, order_id, session_id)).from_err()
}
fn register_order(pool: web::Data<Pool>, params: web::Query<Order>) -> Result<bool, Error> {
    use backend::schema::order_details;
    let conn: &SqliteConnection = &pool.get().unwrap();
    let order_details = OrderDetails {
        order_id: params.order_id.to_string(),
        buyer_public_key: params.buyer_public_key.to_string(),
        buyer_view_key: params.buyer_view_key.to_string(),
        escrow_public_key: params.escrow_public_key.to_string(),
        escrow_view_key: params.escrow_view_key.to_string(),
        session_id: "".to_string(),
    };
    diesel::insert_into(order_details::table)
        .values(&order_details)
        .execute(conn)
        .expect("Error saving new post");
    Ok(true)
}

fn store_session_id(
    pool: web::Data<Pool>,
    order_id_1: String,
    session_id_1: String,
) -> Result<bool, Error> {
    use backend::schema::order_details::dsl::*;
    let conn: &SqliteConnection = &pool.get().unwrap();
    diesel::update(order_details.filter(order_id.eq(&order_id_1)))
        .set(session_id.eq(&session_id_1))
        .execute(conn);
    Ok(true)
}

fn get_order_details(pool: web::Data<Pool>, order_id_1: String) -> Result<OrderDetails, Error> {
    use backend::schema::order_details::dsl::*;
    let conn: &SqliteConnection = &pool.get().unwrap();
    let result = order_details
        .filter(order_id.eq(&order_id_1))
        .first::<OrderDetails>(conn)
        .expect("Error loading posts");
    Ok(result)
}
