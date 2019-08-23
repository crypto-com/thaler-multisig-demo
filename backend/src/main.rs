/*
   Crypto.com Chain Multi-sig backend demo in Actix-Web
*/

use actix_cors::Cors;
use actix_web::{http::header, middleware, web, App, Error as AWError, HttpResponse, HttpServer};
use futures::future::Future;
mod db;
use backend::models::*;
use listenfd::ListenFd;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

use client_common::storage::SledStorage;
use client_common::PublicKey;
use client_core::wallet::DefaultWalletClient;
use client_core::wallet::WalletClient;
use secstr::SecUtf8;
use std::str::FromStr;

use client_core::wallet::MultiSigWalletClient;
use std::str;

use chain_core::tx::data::Tx;
use chain_core::tx::TransactionId;
fn generate_wallet(
    pool: web::Data<Pool>,
    params: web::Query<Order>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    let storage = SledStorage::new(".client-storage").unwrap();
    let wallet = DefaultWalletClient::builder()
        .with_wallet(storage.clone())
        .build()
        .unwrap();
    let passphrase = SecUtf8::from("passphrase");
    let name = params.order_id.to_string();
    wallet.new_wallet(&name, &passphrase).unwrap();
    let public_key = wallet.new_public_key(&name, &passphrase).unwrap();
    let view_key = wallet.view_key(&name, &passphrase).unwrap();
    let keys: Keys = Keys {
        public_key: public_key.to_string(),
        view_key: view_key.to_string(),
    };
    db::execute_register_order(pool, params)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(keys)))
}
fn verify_txid_and_add_commiement(
    pool: web::Data<Pool>,
    params: web::Query<AfterPaid>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    let buyer_commitment_vec = hex::decode(params.commitment.to_string()).unwrap();
    let mut buyer_commitment = [0; 32];
    buyer_commitment.copy_from_slice(&buyer_commitment_vec);

    let storage = SledStorage::new(".client-storage").unwrap();

    let wallet = DefaultWalletClient::builder()
        .with_wallet(storage.clone())
        .build()
        .unwrap();
    let passphrase = SecUtf8::from("passphrase");
    let name = params.order_id.to_string();
    let merchant_public_key = wallet.new_public_key(&name, &passphrase).unwrap();
    db::execute_get_order_details(pool.clone(), name.clone())
        .from_err()
        .and_then(move |res| {
            let buyer_public_key = PublicKey::from_str(&res.buyer_public_key.to_string()).unwrap();
            let transaction = Tx::new();
            let session_id = wallet
                .new_multi_sig_session(
                    &name,
                    &passphrase,
                    transaction.id(),
                    vec![merchant_public_key.clone(), buyer_public_key.clone()],
                    merchant_public_key.clone(),
                )
                .unwrap();
            wallet.add_nonce_commitment(
                &session_id,
                &passphrase,
                buyer_commitment,
                &buyer_public_key,
            );
            let merchant_nonce_commitment =
                wallet.nonce_commitment(&session_id, &passphrase).unwrap();
            let merchant_nonce = wallet.nonce(&session_id, &passphrase).unwrap();
            let afterShipped: AfterShipped = AfterShipped {
                commitment: hex::encode(merchant_nonce_commitment),
                nonce: merchant_nonce.to_string(),
            };
            db::execute_store_session_id(
                pool,
                params.order_id.to_string(),
                hex::encode(&session_id),
            )
            .from_err()
            .and_then(|res| Ok(HttpResponse::Ok().json(afterShipped)))
        })
}
fn add_partial_signature_and_nonce(
    pool: web::Data<Pool>,
    params: web::Query<AfterReceived>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_get_order_details(pool.clone(), name.clone())
        .from_err()
        .and_then(move |res| {
            Ok(HttpResponse::Ok().json(broadcastedTxn))
        })
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
            .service(web::resource("/generate-keys").route(web::post().to_async(generate_wallet)))
            .service(
                web::resource("/submit-commitment-and-nonce")
                    .route(web::post().to_async(verify_txid_and_add_commiement)),
            )
            .service(
                web::resource("/submit-partial-signature-and-nonce")
                    .route(web::post().to_async(add_partial_signature_and_nonce)),
            )
            .service(
                web::resource("/order/pending")
                    .route(web::post().to_async(get_pending_orders)),
            )
    });
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        server.bind("127.0.0.1:8080").unwrap()
    };

    server.run().unwrap();
}
