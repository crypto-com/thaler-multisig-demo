use crate::MultiSigUtxo;
use crate::PartiallySignedTxn;
use actix_web::{web, Error as AWError};
use failure::Error;
use futures::Future;

use diesel;
use diesel::prelude::*;
use backend::*;

pub fn execute_get_partially_signed_transactions(
) -> impl Future<Item = Vec<PartiallySignedTxn>, Error = AWError> {
    web::block(move || get_partially_signed_transactions())
    .from_err()
}
pub fn execute_get_multi_sig_utxos(
) -> impl Future<Item = Vec<MultiSigUtxo>, Error = AWError> {
    web::block(move || get_multi_sig_utxos())
    .from_err()
}
pub fn execute_register_partially_signed_transaction(
    params: web::Query<PartiallySignedTxn>,
) -> impl Future<Item = bool, Error = AWError> {
    web::block(move || register_partially_signed_transaction(params))
    .from_err()
}
pub fn execute_register_multi_sig_utxo(
    params: web::Query<MultiSigUtxo>,
) -> impl Future<Item = bool, Error = AWError> {
    web::block(move || register_multi_sig_utxo(params))
    .from_err()
}
fn get_partially_signed_transactions(
) -> Result<Vec<PartiallySignedTxn>, Error> {

use backend::schema::partially_signed_transaction::dsl::*;
    let connection = establish_connection();
    let results = partially_signed_transaction
        .load::<PartiallySignedTxn>(&connection)
        .expect("Error loading posts");
    Ok(results)
}
fn register_partially_signed_transaction(
    params: web::Query<PartiallySignedTxn>,
) -> Result<bool, Error> {

use backend::schema::partially_signed_transaction;
    let connection = establish_connection();

    let partially_signed_transaction = PartiallySignedTxn { 
        order_id:params.order_id.to_string(), 
        tx_id: params.tx_id.to_string(), 
        output_id:params.output_id,
        hash:params.hash.to_string(), 
        date:params.date.to_string()
     };

    diesel::insert_into(partially_signed_transaction::table)
        .values(&partially_signed_transaction)
        .execute(&connection)
        .expect("Error saving new post");
    Ok(true)
}
fn get_multi_sig_utxos() -> Result<Vec<MultiSigUtxo>, Error> {

    use backend::schema::multi_sig_utxo::dsl::*;
    let connection = establish_connection();
    let results = multi_sig_utxo
        .load::<MultiSigUtxo>(&connection)
        .expect("Error loading posts");
    Ok(results)
}
fn register_multi_sig_utxo(
    params: web::Query<MultiSigUtxo>,
) -> Result<bool, Error> {
    use backend::schema::multi_sig_utxo;
    let connection = establish_connection();
    let multi_sig_utxo = MultiSigUtxo { 
        order_id:params.order_id.to_string(), 
        tx_id: params.tx_id.to_string(), 
        output_id:params.output_id,
        date:params.date.to_string()
     };

    diesel::insert_into(multi_sig_utxo::table)
        .values(&multi_sig_utxo)
        .execute(&connection)
        .expect("Error saving new post");
    Ok(true)

}