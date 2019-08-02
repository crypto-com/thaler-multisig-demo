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


fn get_partially_signed_txns(
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_get_partially_signed_transactions()
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
}

fn register_partially_signed_txn(
    params: web::Query<PartiallySignedTxn>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_register_partially_signed_transaction(params)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
}

fn get_multi_sig_utxos(
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_get_multi_sig_utxos()
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
}

fn register_multi_sig_utxo(
    params: web::Query<MultiSigUtxo>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_register_multi_sig_utxo( params)
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
    let mut server = HttpServer::new(move || {

        App::new()
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
        server.bind("127.0.0.1:8081").unwrap()
    };

    server.run().unwrap();
}
