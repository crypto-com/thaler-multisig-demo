use crate::OrderAndSessionMapping;
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
    session_id: String,
) -> impl Future<Item = bool, Error = AWError> {
    web::block(move || register_order(pool, params, session_id))
    .from_err()
}
fn register_order(
    pool: web::Data<Pool>,
    params: web::Query<Order>,
    session_id: String,
) -> Result<bool, Error> {
    use backend::schema::order_and_session_mapping;
    let conn: &SqliteConnection = &pool.get().unwrap();
    let order_and_session_mapping = OrderAndSessionMapping { 
        order_id:params.order_id.to_string(), 
        session_id: session_id.to_string(), 
     };
    diesel::insert_into(order_and_session_mapping::table)
        .values(&order_and_session_mapping)
        .execute(conn)
        .expect("Error saving new post");
    Ok(true)
}