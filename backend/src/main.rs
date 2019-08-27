/*
   Crypto.com Chain Multi-sig backend demo in Actix-Web
*/
use std::ops::Add;
use simple_error::SimpleError;
use actix_cors::Cors;
use actix_web::{http::header, middleware, web, App, Error as AWError, HttpResponse, HttpServer};
use futures::future::Future;
mod db;
use backend::models::*;
use listenfd::ListenFd;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

use chain_core::common::{H256, HASH_SIZE_256};
use chain_core::init::address::CroAddress;
use chain_core::init::coin::Coin;
use chain_core::tx::data::access::{TxAccess, TxAccessPolicy};
use chain_core::tx::data::attribute::TxAttributes;
use chain_core::tx::data::address::ExtendedAddr;
use chain_core::tx::data::input::TxoPointer;
use chain_core::tx::data::output::TxOut;
use chain_core::tx::data::{Tx, TxId} ;
use chain_core::tx::TransactionId;
use client_common::storage::SledStorage;
use client_common::tendermint::RpcClient;
use client_common::{Transaction, PublicKey};
use client_core::wallet::DefaultWalletClient;
use client_core::wallet::WalletClient;
use client_index::index::{DefaultIndex, Index};
use secstr::SecUtf8;
use std::str::FromStr;

use client_core::wallet::MultiSigWalletClient;

const NETWORK_ID: &str = "AB";
const TENDERMINT_URL: &str = "http://localhost";

// use chain_core::tx::data::Tx;
// use chain_core::tx::TransactionId;
fn new_order(
    pool: web::Data<Pool>,
    params: web::Query<NewOrderRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    let storage = SledStorage::new(".client-storage").unwrap();
    let wallet = DefaultWalletClient::builder()
        .with_wallet(storage.clone())
        .build()
        .unwrap();
    let passphrase = SecUtf8::from("passphrase");
    let name = params.order_id.to_string();
    wallet.new_wallet(&name, &passphrase).unwrap();
    wallet.new_transfer_address(&name, &passphrase).unwrap();
    let ref public_key = &wallet.public_keys(&name, &passphrase).unwrap()[0];
    let view_key = wallet.view_key(&name, &passphrase).unwrap();
    let res = NewOrderResponse {
        public_key: public_key.to_string(),
        view_key: view_key.to_string(),
    };
    db::execute_register_order(pool, params)
        .from_err()
        .and_then(|_| Ok(HttpResponse::Ok().json(res)))
}
fn submit_payment_proof(
    pool: web::Data<Pool>,
    params: web::Query<PaymentProof>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    let order_id = params.order_id.to_string();

    db::execute_get_order_by_id(pool.clone(), order_id.clone())
        .from_err()
        .and_then(move |_| {
            db::execute_store_payment_transaction_id(
                pool,
                params.order_id.to_string(),
                params.transaction_id.to_string(),
            )
            .from_err()
            .and_then(move |_| {
                let res = OrderUpdatedResponse {
                    order_id,
                };
                Ok(HttpResponse::Ok().json(res))
            })
        })
}
fn get_order(
    pool: web::Data<Pool>,
    params: web::Query<OrderRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    let order_id = params.order_id.to_string();

    db::execute_get_order_by_id(pool.clone(), order_id.clone())
        .from_err()
        .and_then(move |record| {
            let storage = SledStorage::new(".client-storage").unwrap();
            let wallet = DefaultWalletClient::builder()
                .with_wallet(storage.clone())
                .build()
                .unwrap();
            let passphrase = SecUtf8::from("passphrase");

            // let nonce_commitment: String = match record.status {
            //     OrderStatus::Delivering | OrderStatus::Refunding => {
            //         let session_id_vec = hex::decode(record.session_id.to_string()).unwrap();
            //         let mut session_id = [0; 32];
            //         session_id.copy_from_slice(&session_id_vec);
            //         let nonce_commitment = wallet.nonce_commitment(&session_id, &passphrase).unwrap();

            //         hex::encode(nonce_commitment)
            //     },
            //     _ => String::from(""),
            // };
            // let nonce: String = match record.status {
            //     OrderStatus::Delivering | OrderStatus::Refunding => {
            //         let session_id_vec = hex::decode(record.session_id.to_string()).unwrap();
            //         let mut session_id = [0; 32];
            //         session_id.copy_from_slice(&session_id_vec);
            //         wallet.nonce(&session_id, &passphrase).unwrap().to_string()
            //     },
            //     _ => String::from(""),
            // };

            let res = OrderResponse {
                order_id: order_id.to_owned(),
                status: record.status,
                price: record.price,
                buyer_public_key: record.buyer_public_key,
                buyer_view_key: record.buyer_view_key,
                buyer_address: record.buyer_address,
                escrow_public_key: record.escrow_public_key,
                escrow_view_key: record.escrow_view_key,
                session_id: record.session_id,
                payment_transaction_id: record.payment_transaction_id,
                settlement_transaction_id: record.settlement_transaction_id,
                // nonce_commitment,
                // nonce
            };
            Ok(HttpResponse::Ok().json(res))
        })
}

fn mark_delivering(
    pool: web::Data<Pool>,
    params: web::Query<OrderRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    mark(pool, params, OrderStatus::Delivering)
}
fn mark_refunding(
    pool: web::Data<Pool>,
    params: web::Query<OrderRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    mark(pool, params, OrderStatus::Refunding)
}
fn mark(
    pool: web::Data<Pool>,
    params: web::Query<OrderRequest>,
    status: OrderStatus,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_get_order_by_id(pool.clone(), params.order_id.clone())
        .from_err()
        .and_then(move |record| {
            db::execute_mark_order_status(pool.clone(), params.order_id.to_string(), status)
                .from_err()
                .and_then(move |_| {
                    let res = OrderUpdatedResponse {
                        order_id: params.order_id.to_string(),
                    };
                    Ok(HttpResponse::Ok().json(res))
                })
        })
}

fn exchange_commitment(
    pool: web::Data<Pool>,
    params: web::Query<ExchangeCommitmentRequest>,
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

    let merchant_public_key = wallet.public_keys(&name, &passphrase).unwrap()[0].clone();
    let merchant_address = wallet.transfer_addresses(&name, &passphrase).unwrap()[0].clone();
    let merchant_view_key = wallet.view_key(&name, &passphrase).unwrap();

    db::execute_get_order_by_id(pool.clone(), name.clone())
        .from_err()
        .and_then(move |record| {
            let buyer_public_key = PublicKey::from_str(&record.buyer_public_key.to_string()).unwrap();
            let buyer_address = ExtendedAddr::from_cro(&record.buyer_address[..]).unwrap();

            let transaction_id_vec = hex::decode(&record.payment_transaction_id).unwrap();
            let mut transaction_id = [0; 32];
            transaction_id.copy_from_slice(&transaction_id_vec);

            let inputs = vec![TxoPointer {
                id: transaction_id,
                index: 0
            }];

            let outputs = match record.status {
                OrderStatus::Delivering => vec![
                    TxOut {
                        address: merchant_address,
                        value: Coin::from_str(&record.price[..]).unwrap(),
                        valid_from: None
                    },
                    TxOut {
                        address: buyer_address,
                        value: Coin::from(10 * 10_000_000),
                        valid_from: None
                    }
                ],
                OrderStatus::Refunding => vec![
                    TxOut {
                        address: buyer_address,
                        value: Coin::from_str(&record.price[..]).unwrap().add(Coin::from(10 * 10_000_000)).unwrap(),
                        valid_from: None
                    }
                ],
                _ => vec![],
            };

            let mut access_policies: Vec<TxAccessPolicy> = vec![];
            let view_keys = vec![
                merchant_view_key,
                PublicKey::from_str(&record.buyer_view_key[..]).unwrap(),
                PublicKey::from_str(&record.escrow_view_key[..]).unwrap(),
            ];
            for key in view_keys.iter() {
                access_policies.push(TxAccessPolicy {
                    view_key: key.into(),
                    access: TxAccess::AllData,
                });
            }

            let network_id = hex::decode("AB").unwrap()[0];
            let attributes = TxAttributes::new_with_access(network_id, access_policies);
            let transaction = Tx {
                inputs,
                outputs,
                attributes,
            };

            let session_id = wallet
                .new_multi_sig_session(
                    &name,
                    &passphrase,
                    transaction.id(),
                    vec![merchant_public_key.clone(), buyer_public_key.clone()],
                    merchant_public_key.clone(),
                )
                .expect("new_multi_sig_session error");

            wallet.add_nonce_commitment(
                &session_id,
                &passphrase,
                buyer_commitment,
                &buyer_public_key,
            ).expect("add_nonce_commitment error");

            let merchant_nonce_commitment =
                wallet.nonce_commitment(&session_id, &passphrase).unwrap();
            let merchant_nonce = wallet.nonce(&session_id, &passphrase).unwrap();

            let res = ExchangeCommitmentResponse {
                order_id: params.order_id.to_string(),
                commitment: hex::encode(merchant_nonce_commitment),
                nonce: merchant_nonce.to_string(),
                transaction_id: hex::encode(transaction.id()),
            };

            db::execute_store_exchanged_data(
                pool,
                params.order_id.to_string(),
                hex::encode(&session_id),
                hex::encode(&transaction.id()),
            )
            .from_err()
            .and_then(|_| Ok(HttpResponse::Ok().json(res)))
        })
}

fn confirm_delivery(
    pool: web::Data<Pool>,
    params: web::Query<ConfirmRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    confirm(pool, params)
}
fn confirm_refund(
    pool: web::Data<Pool>,
    params: web::Query<ConfirmRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    confirm(pool, params)
}
fn confirm(
    pool: web::Data<Pool>,
    params: web::Query<ConfirmRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    let buyer_partial_signature_vec = hex::decode(params.partial_signature.to_string()).unwrap();
    let mut buyer_partial_signature = [0; 32];
    buyer_partial_signature.copy_from_slice(&buyer_partial_signature_vec);

    let buyer_nonce = PublicKey::from_str(&params.nonce.to_string()).unwrap();

    let storage = SledStorage::new(".client-storage").unwrap();
    let wallet = DefaultWalletClient::builder()
        .with_wallet(storage.clone())
        .build()
        .unwrap();
    let passphrase = SecUtf8::from("passphrase");
    let name = params.order_id.to_string();

    db::execute_get_order_by_id(pool.clone(), name.clone())
        .from_err()
        .and_then(move |record| {
            // TODO: Should check if user is confirming the same status as DB record
            let session_id_vec = hex::decode(record.session_id.to_string()).unwrap();
            let mut session_id = [0; 32];
            session_id.copy_from_slice(&session_id_vec);
            let buyer_public_key = PublicKey::from_str(&record.buyer_public_key.to_string()).unwrap();

            wallet.add_nonce(&session_id, &passphrase, &buyer_nonce, &buyer_public_key);

            wallet.add_partial_signature(
                &session_id,
                &passphrase,
                buyer_partial_signature,
                &buyer_public_key,
            );

            wallet.signature(&session_id, &passphrase).unwrap();

            let res = ConfirmResponse {
                transaction_id: record.settlement_transaction_id.to_string(),
            };
            Ok(HttpResponse::Ok().json(res))
        })
}

fn get_pending_payment_orders(
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_get_orders_by_status(pool.clone(), vec![OrderStatus::PendingPayment])
        .from_err()
        .and_then(move |res| {
            Ok(HttpResponse::Ok().json(res))
        })
}
fn get_pending_response_orders(
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_get_orders_by_status(pool.clone(), vec![OrderStatus::Paid])
        .from_err()
        .and_then(move |res| {
            Ok(HttpResponse::Ok().json(res))
        })
}
fn get_settled_orders(
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_get_orders_by_status(pool.clone(), vec![OrderStatus::Paid, OrderStatus::Refunded])
        .from_err()
        .and_then(move |res| {
            Ok(HttpResponse::Ok().json(res))
        })
}

fn get_transaction_by_id(transaction_id: String, order_id: String) -> Result<Option<Transaction>, SimpleError>{
    let tendermint_client = RpcClient::new(TENDERMINT_URL);
    let storage = SledStorage::new(".client-storage").unwrap();
    let index = DefaultIndex::new(storage.clone(), tendermint_client);
    let wallet = DefaultWalletClient::builder()
        .with_wallet(storage.clone())
        .build()
        .unwrap();
    let passphrase = SecUtf8::from("passphrase");
    let name = order_id.to_string();

    let transaction_id_vec = hex::decode(transaction_id).unwrap();
    if transaction_id_vec.len() != 32 {
        return Err(SimpleError::new(format!(
            "Invalid transaction id length: {}",
            transaction_id_vec.len()
        )));
    }
    let mut transaction_id = [0; 32];
    transaction_id.copy_from_slice(&transaction_id_vec);

    let transaction_id: &TxId = &transaction_id;
    index.transaction(transaction_id).map_err(|err| SimpleError::new(format!("{}", err)))
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
            .service(web::resource("/order/new").route(web::post().to_async(new_order)))
            .service(
                web::resource("/order/payment-proof")
                    .route(web::post().to_async(submit_payment_proof)),
            )
            .service(
                web::resource("/order")
                    .route(web::get().to_async(get_order)),
            )
            .service(
                web::resource("/order/delivering")
                    .route(web::post().to_async(mark_delivering)),
            )
            .service(
                web::resource("/order/refunding")
                    .route(web::post().to_async(mark_refunding)),
            )
            .service(
                web::resource("/order/exchange-commitment")
                    .route(web::post().to_async(mark_refunding)),
            )
            .service(
                web::resource("/order/confirm/delivery")
                    .route(web::post().to_async(confirm_delivery)),
            )
            .service(
                web::resource("/order/confirm/refund")
                    .route(web::post().to_async(confirm_refund)),
            )
            .service(
                web::resource("/order/pending-payment")
                    .route(web::get().to_async(get_pending_payment_orders)),
            )
            .service(
                web::resource("/order/pending-response")
                    .route(web::get().to_async(get_pending_response_orders)),
            )
            .service(
                web::resource("/order/settled")
                    .route(web::get().to_async(get_settled_orders)),
            )
    });
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        server.bind("127.0.0.1:8080").unwrap()
    };

    server.run().unwrap();
}