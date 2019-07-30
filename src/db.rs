use crate::MultiSigUtxo;
use crate::PartiallySignedTxn;
use actix_web::{web, Error as AWError};
use failure::Error;
use futures::Future;
use r2d2;
use r2d2_sqlite;
use rusqlite::NO_PARAMS;
pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

pub fn execute_get_partially_signed_transactions(
    pool: &Pool,
) -> impl Future<Item = Vec<PartiallySignedTxn>, Error = AWError> {
    let pool = pool.clone();
    web::block(move || get_partially_signed_transactions(pool.get()?))
    .from_err()
}
pub fn execute_get_multi_sig_utxos(
    pool: &Pool,
) -> impl Future<Item = Vec<MultiSigUtxo>, Error = AWError> {
    let pool = pool.clone();
    web::block(move || get_multi_sig_utxos(pool.get()?))
    .from_err()
}
pub fn execute_register_partially_signed_transaction(
    pool: &Pool,
    params: web::Query<PartiallySignedTxn>,
) -> impl Future<Item = bool, Error = AWError> {
    let pool = pool.clone();
    web::block(move || register_partially_signed_transaction(pool.get()?, params))
    .from_err()
}
pub fn execute_register_multi_sig_utxo(
    pool: &Pool,
    params: web::Query<MultiSigUtxo>,
) -> impl Future<Item = bool, Error = AWError> {
    let pool = pool.clone();
    web::block(move || register_multi_sig_utxo(pool.get()?, params))
    .from_err()
}
fn get_partially_signed_transactions(
    conn: Connection,
) -> Result<Vec<PartiallySignedTxn>, Error> {

    let stmt = "SELECT ORDER_ID, TX_ID, OUTPUT_ID,HASH, DATE
    FROM partially_signed_transaction;";

    let mut prep_stmt = conn.prepare(stmt)?;

    let txns = prep_stmt
        .query_map(NO_PARAMS, |row| PartiallySignedTxn {
            order_id: row.get(0),
            tx_id: row.get(1),
            output_id: row.get(2),
            hash: row.get(3),
            date: row.get(4),
        })
        .and_then(|mapped_rows| {
            Ok(mapped_rows
                .map(|row| row.unwrap())
                .collect::<Vec<PartiallySignedTxn>>())
        })?;

    Ok(txns)
}
fn register_partially_signed_transaction(
    conn: Connection,
    params: web::Query<PartiallySignedTxn>,
) -> Result<bool, Error> {

    conn.execute(
        "INSERT INTO partially_signed_transaction (ORDER_ID, TX_ID, OUTPUT_ID,HASH, DATE)
                  VALUES (?1, ?2, ?3, ?4, ?5)",
        &[&params.order_id.to_string(), &params.tx_id.to_string(), &params.output_id.to_string(),&params.hash.to_string(),&params.date.to_string()],
    )
    .unwrap();
    Ok(true)
}
fn get_multi_sig_utxos(conn: Connection) -> Result<Vec<MultiSigUtxo>, Error> {
    let stmt = "SELECT ORDER_ID, TX_ID, OUTPUT_ID, DATE
    FROM multi_sig_utxo;";

    let mut prep_stmt = conn.prepare(stmt)?;

    let utxos = prep_stmt
        .query_map(NO_PARAMS, |row| MultiSigUtxo {
            order_id: row.get(0),
            tx_id: row.get(1),
            output_id: row.get(2),
            date: row.get(3),
        })
        .and_then(|mapped_rows| {
            Ok(mapped_rows
                .map(|row| row.unwrap())
                .collect::<Vec<MultiSigUtxo>>())
        })?;

    Ok(utxos)
}
fn register_multi_sig_utxo(
    conn: Connection,
    params: web::Query<MultiSigUtxo>,
) -> Result<bool, Error> {

    conn.execute(
        "INSERT INTO multi_sig_utxo (ORDER_ID, TX_ID, OUTPUT_ID, DATE)
                  VALUES (?1, ?2, ?3, ?4)",
        &[
            &params.order_id.to_string(),
            &params.tx_id.to_string(),
            &params.output_id.to_string(),
            &params.date.to_string(),
        ],
    )
    .unwrap();
    Ok(true)
}