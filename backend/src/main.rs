/*
   Crypto.com Chain Multi-sig backend demo in Actix-Web
*/

use actix_web::Error;
use actix_cors::Cors;
use actix_web::{http::header,middleware, web, App, Error as AWError, HttpResponse, HttpServer};
use futures::future::Future;
mod db;
use listenfd::ListenFd;
use backend::models::*;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

fn get_partially_signed_txns(
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_get_partially_signed_transactions(pool)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
}

fn register_partially_signed_txn(
    pool: web::Data<Pool>,
    params: web::Query<PartiallySignedTxn>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_register_partially_signed_transaction(pool, params)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
}

fn get_multi_sig_utxos(
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_get_multi_sig_utxos(pool)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
}

fn register_multi_sig_utxo(
    pool: web::Data<Pool>,
    params: web::Query<MultiSigUtxo>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_register_multi_sig_utxo(pool, params)
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
    dotenv::dotenv().ok();
    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<SqliteConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let mut server = HttpServer::new(move || {

        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600),
            )
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
