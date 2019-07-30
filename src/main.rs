/*
   Crypto.com Chain Multi-sig backend demo in Actix-Web
*/

use actix_web::Error;
use actix_web::{middleware, web, App, Error as AWError, HttpResponse, HttpServer};
use futures::future::Future;
use r2d2_sqlite;
use r2d2_sqlite::SqliteConnectionManager;
use serde::{Deserialize, Serialize}; 
mod db;
use db::{Pool};
use listenfd::ListenFd;

fn get_partially_signed_txns(
    db: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_get_partially_signed_transactions(&db)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
}

#[derive(Serialize, Deserialize)]
pub struct PartiallySignedTxn {
    order_id: String,
    tx_id: String,
    output_id: i32,
    hash: String,
    date: String,
}

#[derive(Serialize, Deserialize)]
pub struct MultiSigUtxo {
    order_id: String,
    tx_id: String,
    output_id: i32,
    date: String,
}
#[derive(Serialize)]
pub struct Address {
    address: String,
}
fn register_partially_signed_txn(
    params: web::Query<PartiallySignedTxn>,
    db: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_register_partially_signed_transaction(&db, params)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
}

fn get_multi_sig_utxos(
    db: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_get_multi_sig_utxos(&db)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
}

fn register_multi_sig_utxo(
    params: web::Query<MultiSigUtxo>,
    db: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_register_multi_sig_utxo(&db, params)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
}

fn get_to_addresses() -> Result<HttpResponse, Error> {
    return_dummy_address()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
}

fn return_dummy_address(
) -> Result<Address, Error> {
    Ok(Address{address:"0xaddress".to_string()})
}

fn main() {
    let mut listenfd = ListenFd::from_env();
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let manager = SqliteConnectionManager::file("multi-sig.db");
    let pool = Pool::new(manager).unwrap();
    let mut server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/transaction/partially-signed")
                    .route(web::get().to_async(get_partially_signed_txns))
                    .route(web::post().to_async(register_partially_signed_txn)),
            )
            .service(
                web::resource("/multi-sig-utxo")
                    .route(web::get().to_async(get_multi_sig_utxos))
                    .route(web::post().to_async(register_multi_sig_utxo)),
            )
            .service(
                web::resource("/address/merchant/{merchantId}/order-id/{orderId}")
                    .route(web::get().to_async(get_to_addresses)),
            )
    });
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        server.bind("127.0.0.1:8080").unwrap()
    };

    server.run().unwrap();
}
