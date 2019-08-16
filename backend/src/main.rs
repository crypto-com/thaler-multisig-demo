/*
   Crypto.com Chain Multi-sig backend demo in Actix-Web
*/

use actix_cors::Cors;
use actix_web::{http::header,middleware, web, App, Error as AWError, HttpResponse, HttpServer};
use futures::future::Future;
mod db;
use listenfd::ListenFd;
use backend::models::*;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

use client_common::storage::MemoryStorage;
use client_core::wallet::DefaultWalletClient;
use client_core::wallet::WalletClient;
use secstr::SecUtf8;
fn generate_wallet(
    pool: web::Data<Pool>,
    params: web::Query<Order>,
) -> impl Future<Item = HttpResponse, Error = AWError> {

    let storage = MemoryStorage::default();
    let wallet = DefaultWalletClient::builder()
    .with_wallet(storage.clone())
    .build()
    .unwrap();
    let passphrase = &SecUtf8::from("passphrase");
    let name = params.order_id.to_string();
    wallet.new_wallet(&name, passphrase).unwrap();
    let public_key = wallet.new_public_key(&name, passphrase).unwrap();
    let view_key = wallet.view_key(&name, passphrase).unwrap();
    let keys:Keys = Keys{pub_key:public_key.to_string(),view_key:view_key.to_string()};
    db::execute_register_order(pool, params)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(keys)))
}
fn verify_txid_and_add_commiement(
        pool: web::Data<Pool>,
    params: web::Query<AfterPaid>,
) -> Result<HttpResponse, AWError>{
    // verify txid
    // create sessionM
    // add commitmentB to sessionM
    // generate commitmentM and nonceM
    // return commitmentM and nonceM
    let afterShipped:AfterShipped = AfterShipped{ commitment:"commitment".to_string(), nonce:"nonce".to_string()};
    Ok(HttpResponse::Ok().json(afterShipped))
}
fn update_signed_txn_and_nonce(
        pool: web::Data<Pool>,
    params: web::Query<AfterReceived>,
) -> Result<HttpResponse, AWError>{
    // signedTxnB and nonceB
    // add nonceB to sessionM
    // add signTxnB to seesionM
    // sign sessionM
    // broadcast txn
    let broadcastedTxn:BroadcastedTxn = BroadcastedTxn{tx_id:"tx_id".to_string()};
    Ok(HttpResponse::Ok().json(broadcastedTxn))
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
                web::resource("/generate-keys")
                    .route(web::post().to_async(generate_wallet)),
            )
            .service(
                web::resource("/submit-commitment-and-nonce")
                    .route(web::post().to_async(verify_txid_and_add_commiement)),
            )
            .service(
                web::resource("/submit-signed-txn-and-nounce")
                    .route(web::post().to_async(update_signed_txn_and_nonce)),
            )
    });
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        server.bind("127.0.0.1:8080").unwrap()
    };

    server.run().unwrap();
}
