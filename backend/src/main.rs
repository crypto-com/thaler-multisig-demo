/*
   Crypto.com Chain Multi-sig backend demo in Actix-Web
*/

// use actix_web::Error;
use actix_cors::Cors;
use actix_web::{http::header,middleware, web, App, Error as AWError, HttpResponse, HttpServer};
use futures::future::Future;
mod db;
use listenfd::ListenFd;
use backend::models::*;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;


fn generate_wallet(
    pool: web::Data<Pool>,
    params: web::Query<Order>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    // generate wallet from client
    // get pubkeyM, viewkeyM and session_idM from client
    let session_id = "session_id";
    let pub_key = "pub_key";
    let view_key = "view_key";
    let keys:Keys = Keys{pub_key:pub_key.to_string(),view_key:view_key.to_string()};
    db::execute_register_order(pool, params, session_id.to_string())
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(keys)))
}
fn verify_txid_and_add_commiement(
        pool: web::Data<Pool>,
    params: web::Query<Order>,
) -> Result<HttpResponse, AWError>{
    // txid and commitmentB
    // add commitmentB to session_idM in client
    // generate commitmentM and nonceM from client
    // return commitmentM and nonceM

    Ok(HttpResponse::Ok().json(true))
}
fn update_signed_txn_and_nonce(
        pool: web::Data<Pool>,
    params: web::Query<Order>,
) -> Result<HttpResponse, AWError>{
    // result B and nonceB
    // ask client to add nonceB to client
    // ask client to sign seesionM
    // ask client to add resultB
    // broadcast txn
    // return txid
    Ok(HttpResponse::Ok().json(true))
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
                web::resource("/keys")
                    .route(web::post().to_async(generate_wallet)),
                    // return pubkey and view key
            )
            .service(
                web::resource("/commitment-and-nonce/order-id/{orderId}/txId/{txId}/commitment/{commitment}")
                    .route(web::get().to_async(verify_txid_and_add_commiement)),
                    // return commitment and nonce
            )
            .service(
                web::resource("/partially-signed-transaction/{signedTransaction}/order-id/{orderId}/nonce/{nonce}/")
                    .route(web::get().to_async(update_signed_txn_and_nonce)),
            )
    });
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        server.bind("127.0.0.1:8080").unwrap()
    };

    server.run().unwrap();
}
