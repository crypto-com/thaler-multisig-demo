use crate::OrderDetails;
use crate::Order;
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
    web::block(move || register_order(pool, params))
    .from_err()
}
fn register_order(
    pool: web::Data<Pool>,
    params: web::Query<Order>,
) -> Result<bool, Error> {
    use backend::schema::order_details;
    let conn: &SqliteConnection = &pool.get().unwrap();
    let order_details = OrderDetails { 
        order_id:params.order_id.to_string(), 
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