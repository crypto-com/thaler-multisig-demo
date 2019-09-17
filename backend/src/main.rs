/*
   Crypto.com Chain Multi-sig backend demo in Actix-Web
*/
#[macro_use]
extern crate diesel;

use actix_cors::Cors;
use actix_web::{http::header, middleware, web, App, Error as AWError, HttpResponse, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use futures::future::Future;
use listenfd::ListenFd;
use secstr::SecUtf8;
use std::ops::Sub;
use std::str::FromStr;
use uuid::Uuid;

use chain_core::init::address::CroAddress;
use chain_core::init::coin::Coin;
use chain_core::tx::data::access::{TxAccess, TxAccessPolicy};
use chain_core::tx::data::address::ExtendedAddr;
use chain_core::tx::data::attribute::TxAttributes;
use chain_core::tx::data::input::TxoPointer;
use chain_core::tx::data::output::TxOut;
use chain_core::tx::data::{Tx, TxId};
use chain_core::tx::fee::LinearFee;
use chain_core::tx::TransactionId;
use client_common::storage::SledStorage;
use client_common::tendermint::{Client, RpcClient};
use client_common::{PublicKey, Transaction};
use client_core::signer::DefaultSigner;
use client_core::transaction_builder::DefaultTransactionBuilder;
use client_core::wallet::{DefaultWalletClient, MultiSigWalletClient, WalletClient};
use client_index::cipher::MockAbciTransactionObfuscation;
use client_index::handler::{DefaultBlockHandler, DefaultTransactionHandler};
use client_index::index::{DefaultIndex, Index};
use client_index::synchronizer::ManualSynchronizer;

use crate::models::*;

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

mod db;
mod models;
mod schema;

const NETWORK_ID: &str = "42";
const TENDERMINT_URL: &str = "http://localhost:26657";

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
            .service(web::resource("/order").route(web::get().to_async(get_order)))
            .service(
                web::resource("/order/delivering").route(web::post().to_async(mark_delivering)),
            )
            .service(web::resource("/order/refunding").route(web::post().to_async(mark_refunding)))
            .service(
                web::resource("/order/exchange-commitment")
                    .route(web::post().to_async(exchange_commitment)),
            )
            .service(
                web::resource("/order/confirm/delivery")
                    .route(web::post().to_async(confirm_delivery)),
            )
            .service(
                web::resource("/order/confirm/refund").route(web::post().to_async(confirm_refund)),
            )
            .service(web::resource("/order/pending").route(web::get().to_async(get_pending_orders)))
            .service(
                web::resource("/order/outstanding")
                    .route(web::get().to_async(get_pending_response_orders)),
            )
            .service(
                web::resource("/order/completed").route(web::get().to_async(get_settled_orders)),
            )
    });
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        server.bind("0.0.0.0:8080").unwrap()
    };

    server.run().unwrap();
}

fn new_order(
    pool: web::Data<Pool>,
    params: web::Form<NewOrderRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_is_order_exist(pool.clone(), params.order_id.to_string())
        .and_then(|exist| {
            if exist {
                return Err(AWError::from(
                    HttpResponse::BadRequest()
                        .reason("Order already exist")
                        .finish(),
                ));
            }
            Ok(())
        })
        .and_then(move |_| {
            let (wallet, _, _) = make_app();
            let wallet_name = Uuid::new_v4().to_string();
            let passphrase = SecUtf8::from("passphrase");

            wallet
                .new_wallet(&wallet_name, &passphrase)
                .expect("new_wallet error");

            let merchant_address = wallet
                .new_transfer_address(&wallet_name, &passphrase)
                .unwrap();
            let merchant_public_key = &wallet.public_keys(&wallet_name, &passphrase).unwrap()[0];
            let merchant_public_key = merchant_public_key.to_owned();
            let merchant_view_key = wallet.view_key(&wallet_name, &passphrase).unwrap();

            let buyer_public_key =
                PublicKey::from_str(&params.buyer_public_key.to_string()).unwrap();
            let escrow_public_key =
                PublicKey::from_str(&params.escrow_public_key.to_string()).unwrap();

            let multisig_address = wallet
                .new_multisig_transfer_address(
                    &wallet_name,
                    &passphrase,
                    vec![
                        merchant_public_key.clone(),
                        buyer_public_key,
                        escrow_public_key,
                    ],
                    merchant_public_key.clone(),
                    2,
                    3,
                )
                .expect("new_multisig_transfer_address error");

            let order = Order {
                order_id: params.order_id.to_string(),
                amount: params.amount.to_string(),
                wallet_name: wallet_name.clone(),
                status: OrderStatus::PendingPayment,
                buyer_public_key: params.buyer_public_key.to_string(),
                buyer_view_key: params.buyer_view_key.to_string(),
                buyer_address: params.buyer_address.to_string(),
                escrow_public_key: params.escrow_public_key.to_string(),
                escrow_view_key: params.escrow_view_key.to_string(),
                session_id: "".to_string(),
                payment_transaction_id: "".to_string(),
                settlement_transaction_id: "".to_string(),
            };

            let res = NewOrderResponse {
                public_key: merchant_public_key.to_string(),
                address: merchant_address.to_string(),
                view_key: merchant_view_key.to_string(),
                multisig_address: multisig_address.to_string(),
            };

            db::execute_register_order(pool, order)
                .from_err()
                .and_then(|_| Ok(HttpResponse::Ok().json(res)))
        })
}

fn submit_payment_proof(
    pool: web::Data<Pool>,
    params: web::Form<PaymentProof>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    // TODO: Consider using Arc to share resource
    let query_order_id = params.order_id.to_string();
    let query_transaction_id = params.transaction_id.to_string();
    let query_pool = pool.clone();

    let update_order_id = params.order_id.to_string();
    let update_transaction_id = params.transaction_id.to_string();
    let update_pool = pool.clone();

    let return_order_id = params.order_id.to_string();

    db::execute_is_order_exist(query_pool.clone(), query_order_id.clone())
        .and_then(|exist| {
            if !exist {
                return Err(AWError::from(HttpResponse::NotFound().finish()));
            }
            Ok(())
        })
        .and_then(move |_| db::execute_get_order_by_id(query_pool, query_order_id))
        .and_then(move |record| {
            if record.status != OrderStatus::PendingPayment {
                return Err(AWError::from(
                    HttpResponse::BadRequest()
                        .reason("Transaction Not Pending for Payment")
                        .finish(),
                ));
            }

            let transaction =
                get_transaction_by_id(query_transaction_id, record.wallet_name.clone());
            let transaction = match transaction {
                None => {
                    return Err(AWError::from(
                        HttpResponse::NotFound()
                            .reason("Transaction Not Found")
                            .finish(),
                    ))
                }
                Some(transaction) => transaction,
            };

            if let Transaction::TransferTransaction(tx) = transaction {
                Ok((tx, record))
            } else {
                Err(AWError::from(
                    HttpResponse::BadRequest()
                        .reason("Invalid Transaction Type")
                        .finish(),
                ))
            }
        })
        .and_then(move |(tx, record)| {
            let (wallet, _, _) = make_app();
            let wallet_name = &record.wallet_name.to_string();
            let passphrase = SecUtf8::from("passphrase");

            if tx.outputs.len() == 0 {
                return Err(AWError::from(
                    HttpResponse::BadRequest()
                        .reason("Transaction Has No Output")
                        .finish(),
                ));
            }

            let merchant_public_key = &wallet.public_keys(&wallet_name, &passphrase).unwrap()[0];
            let merchant_public_key = merchant_public_key.to_owned();
            let buyer_public_key =
                PublicKey::from_str(&record.buyer_public_key.to_string()).unwrap();
            let escrow_public_key =
                PublicKey::from_str(&record.escrow_public_key.to_string()).unwrap();
            let multisig_address = wallet
                .new_multisig_transfer_address(
                    &wallet_name,
                    &passphrase,
                    vec![
                        merchant_public_key.clone(),
                        buyer_public_key.clone(),
                        escrow_public_key.clone(),
                    ],
                    merchant_public_key.clone(),
                    2,
                    3,
                )
                .unwrap();
            if tx.outputs[0].address.to_cro().unwrap() != multisig_address.to_string() {
                return Err(AWError::from(
                    HttpResponse::BadRequest()
                        .reason("Incorrect Transaction Output Address")
                        .finish(),
                ));
            }
            if tx.outputs[0].value != Coin::from_str(&record.amount).unwrap() {
                return Err(AWError::from(
                    HttpResponse::BadRequest()
                        .reason("Incorrect Transaction Output Amount")
                        .finish(),
                ));
            }
            Ok(())
        })
        .and_then(move |_| {
            db::execute_store_payment_transaction_id(
                update_pool,
                update_order_id,
                update_transaction_id,
            )
        })
        .and_then(move |_| {
            let res = OrderUpdatedResponse {
                order_id: return_order_id,
            };
            Ok(HttpResponse::Ok().json(res))
        })
}

fn get_order(
    pool: web::Data<Pool>,
    params: web::Query<OrderRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    // TODO: Consider using Arc to share resource
    let query_order_id = params.order_id.to_string();
    let query_pool = pool.clone();

    let return_order_id = params.order_id.to_string();

    db::execute_is_order_exist(query_pool.clone(), query_order_id.clone())
        .and_then(|exist| {
            if !exist {
                return Err(AWError::from(HttpResponse::NotFound().finish()));
            }
            Ok(())
        })
        .and_then(move |_| db::execute_get_order_by_id(query_pool, query_order_id))
        .and_then(move |record| {
            // Uncomment to return commitment and nonce in response
            // let (wallet, _, _) = make_app();
            // let passphrase = SecUtf8::from("passphrase");

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
                order_id: return_order_id,
                status: record.status,
                amount: record.amount,
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
    params: web::Form<OrderRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    mark(pool, params, OrderStatus::Delivering)
}
fn mark_refunding(
    pool: web::Data<Pool>,
    params: web::Form<OrderRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    mark(pool, params, OrderStatus::Refunding)
}
fn mark(
    pool: web::Data<Pool>,
    params: web::Form<OrderRequest>,
    status: OrderStatus,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    // TODO: Consider using Arc to share resource
    let query_order_id = params.order_id.to_string();
    let query_pool = pool.clone();

    let update_order_id = params.order_id.to_string();
    let update_pool = pool.clone();

    let return_order_id = params.order_id.to_string();

    db::execute_is_order_exist(query_pool.clone(), query_order_id.clone())
        .and_then(|exist| {
            if !exist {
                return Err(AWError::from(HttpResponse::NotFound().finish()));
            }
            Ok(())
        })
        .and_then(move |_| db::execute_get_order_by_id(query_pool, query_order_id))
        .and_then(move |_| {
            // TODO: Check status
            db::execute_update_order_status(update_pool, update_order_id.clone(), status)
        })
        .and_then(move |_| {
            let res = OrderUpdatedResponse {
                order_id: return_order_id,
            };
            Ok(HttpResponse::Ok().json(res))
        })
}

fn exchange_commitment(
    pool: web::Data<Pool>,
    params: web::Form<ExchangeCommitmentRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    let buyer_commitment_vec = hex::decode(params.commitment.to_string()).unwrap();
    let mut buyer_commitment = [0; 32];
    buyer_commitment.copy_from_slice(&buyer_commitment_vec);

    let (wallet, _, _) = make_app();
    let passphrase = SecUtf8::from("passphrase");

    // TODO: Consider using Arc to share resource
    let query_order_id = params.order_id.to_string();
    let query_pool = pool.clone();

    let update_order_id = params.order_id.to_string();
    let update_pool = pool.clone();

    let return_order_id = params.order_id.to_string();

    db::execute_is_order_exist(query_pool.clone(), query_order_id.clone())
        .and_then(|exist| {
            if !exist {
                return Err(AWError::from(HttpResponse::NotFound().finish()));
            }
            Ok(())
        })
        .and_then(move |_| db::execute_get_order_by_id(query_pool, query_order_id))
        .and_then(move |record| {
            if record.status != OrderStatus::Delivering && record.status != OrderStatus::Refunding {
                return Err(AWError::from(
                    HttpResponse::BadRequest()
                        .reason("Transaction Not Ready")
                        .finish(),
                ));
            }
            // TODO: Check order hasn't exchanged commitment before

            Ok(record)
        })
        .and_then(move |record| {
            let wallet_name = record.wallet_name.clone();

            let merchant_public_key =
                wallet.public_keys(&wallet_name, &passphrase).unwrap()[0].clone();
            let buyer_public_key =
                PublicKey::from_str(&record.buyer_public_key.to_string()).unwrap();

            let transaction =
                construct_tx(wallet_name.clone(), passphrase.clone(), &wallet, &record);

            let session_id = wallet
                .new_multi_sig_session(
                    &wallet_name,
                    &passphrase,
                    transaction.id(),
                    vec![merchant_public_key.clone(), buyer_public_key.clone()],
                    merchant_public_key.clone(),
                )
                .expect("new_multi_sig_session error");

            // TODO: Handle duplicate add error
            wallet
                .add_nonce_commitment(
                    &session_id,
                    &passphrase,
                    buyer_commitment,
                    &buyer_public_key,
                )
                .expect("add_nonce_commitment error");

            let merchant_nonce_commitment = wallet
                .nonce_commitment(&session_id, &passphrase)
                .expect("nonce_commitment error");
            let merchant_nonce = wallet.nonce(&session_id, &passphrase).expect("nonce error");

            let res = ExchangeCommitmentResponse {
                order_id: return_order_id,
                commitment: hex::encode(merchant_nonce_commitment),
                nonce: merchant_nonce.to_string(),
                transaction_id: hex::encode(transaction.id()),
                transaction: transaction.clone(),
            };

            db::execute_store_exchanged_data(
                update_pool,
                update_order_id,
                hex::encode(&session_id),
                hex::encode(&transaction.id()),
            )
            .from_err()
            .and_then(|_| Ok(HttpResponse::Ok().json(res)))
        })
}

fn confirm_delivery(
    pool: web::Data<Pool>,
    params: web::Form<ConfirmRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    // TODO: Should check order is in delivering status
    confirm(pool, params, OrderStatus::Completed)
}
fn confirm_refund(
    pool: web::Data<Pool>,
    params: web::Form<ConfirmRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    // TODO: Should check order is in refunding status
    confirm(pool, params, OrderStatus::Refunded)
}
fn confirm(
    pool: web::Data<Pool>,
    params: web::Form<ConfirmRequest>,
    status: OrderStatus,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    let buyer_partial_signature_vec = hex::decode(params.partial_signature.to_string()).unwrap();
    let mut buyer_partial_signature = [0; 32];
    buyer_partial_signature.copy_from_slice(&buyer_partial_signature_vec);

    let buyer_nonce = PublicKey::from_str(&params.nonce.to_string()).unwrap();

    let (wallet, _, synchronizer) = make_app();
    let passphrase = SecUtf8::from("passphrase");

    // TODO: Consider using Arc to share resource

    let query_order_id = params.order_id.to_string();
    let query_pool = pool.clone();

    let update_order_id = params.order_id.to_string();
    let update_pool = pool.clone();

    let return_order_id = params.order_id.to_string();

    db::execute_is_order_exist(query_pool.clone(), query_order_id.clone())
        .and_then(|exist| {
            if !exist {
                return Err(AWError::from(HttpResponse::NotFound().finish()));
            }
            Ok(())
        })
        .and_then(move |_| db::execute_get_order_by_id(query_pool, query_order_id))
        .and_then(move |record| {
            match status {
                OrderStatus::Completed => {
                    if record.status != OrderStatus::Delivering {
                        return Err(AWError::from(
                            HttpResponse::BadRequest()
                                .reason("Refunding Transaction Cannot Confirm Delivery")
                                .finish(),
                        ));
                    }
                }
                OrderStatus::Refunded => {
                    if record.status != OrderStatus::Refunding {
                        return Err(AWError::from(
                            HttpResponse::BadRequest()
                                .reason("Delivering Transaction Cannot Refund")
                                .finish(),
                        ));
                    }
                }
                _ => {
                    return Err(AWError::from(HttpResponse::InternalServerError().finish()));
                }
            }

            let wallet_name = record.wallet_name.clone();

            // Complete multi-sig session
            let session_id_vec = hex::decode(record.session_id.to_string()).unwrap();
            let mut session_id = [0; 32];
            session_id.copy_from_slice(&session_id_vec);
            let buyer_public_key =
                PublicKey::from_str(&record.buyer_public_key.to_string()).unwrap();

            // TODO: Handle duplicate add error
            wallet
                .add_nonce(&session_id, &passphrase, &buyer_nonce, &buyer_public_key)
                .expect("add_nonce error");

            wallet
                .partial_signature(&session_id, &passphrase)
                .expect("partial_signature error");

            // TODO: Handle duplicate add error
            wallet
                .add_partial_signature(
                    &session_id,
                    &passphrase,
                    buyer_partial_signature,
                    &buyer_public_key,
                )
                .expect("add_partial_signature error");

            wallet
                .signature(&session_id, &passphrase)
                .expect("signature error");

            let merchant_view_key = wallet.view_key(&wallet_name, &passphrase).unwrap();

            let transaction_id_vec = hex::decode(&record.payment_transaction_id).unwrap();
            let mut transaction_id = [0; 32];
            transaction_id.copy_from_slice(&transaction_id_vec);

            let merchant_private_key = wallet
                .private_key(&passphrase, &merchant_view_key)
                .unwrap()
                .unwrap();
            let merchant_staking_addresses =
                wallet.staking_addresses(&wallet_name, &passphrase).unwrap();
            // TODO: Create separate thread to sync in background
            synchronizer
                .sync(
                    &merchant_staking_addresses,
                    &merchant_view_key,
                    &merchant_private_key,
                    None,
                    None,
                )
                .expect("sync error");

            let transaction =
                construct_tx(wallet_name.clone(), passphrase.clone(), &wallet, &record);

            let tx_aux = wallet
                .transaction(&wallet_name, &session_id, &passphrase, transaction)
                .expect("transaction error");

            wallet
                .broadcast_transaction(&tx_aux)
                .expect("broadcast_transaction error");
            Ok(record)
        })
        .and_then(move |record| {
            db::execute_update_order_status(update_pool, update_order_id.clone(), status).and_then(
                move |_| {
                    let res = ConfirmResponse {
                        order_id: return_order_id,
                        transaction_id: record.settlement_transaction_id.to_string(),
                    };
                    Ok(HttpResponse::Ok().json(res))
                },
            )
        })
}

fn get_pending_orders(pool: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_get_orders_by_status(
        pool.clone(),
        vec![
            OrderStatus::PendingPayment,
            OrderStatus::Delivering,
            OrderStatus::Refunding,
        ],
    )
    .from_err()
    .and_then(move |res| Ok(HttpResponse::Ok().json(res)))
}

fn get_pending_response_orders(
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_get_orders_by_status(pool.clone(), vec![OrderStatus::PendingResponse])
        .from_err()
        .and_then(move |res| Ok(HttpResponse::Ok().json(res)))
}

fn get_settled_orders(pool: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = AWError> {
    db::execute_get_orders_by_status(
        pool.clone(),
        vec![OrderStatus::Completed, OrderStatus::Refunded],
    )
    .from_err()
    .and_then(move |res| Ok(HttpResponse::Ok().json(res)))
}

fn get_transaction_by_id(transaction_id: String, wallet_name: String) -> Option<Transaction> {
    let (wallet, index, synchronizer) = make_app();
    let passphrase = SecUtf8::from("passphrase");

    let merchant_view_key = wallet.view_key(&wallet_name, &passphrase).unwrap();
    let merchant_private_key = wallet
        .private_key(&passphrase, &merchant_view_key)
        .unwrap()
        .unwrap();
    let merchant_staking_addresses = wallet.staking_addresses(&wallet_name, &passphrase).unwrap();

    // TODO: Create separate thread to sync in background
    synchronizer
        .sync(
            &merchant_staking_addresses,
            &merchant_view_key,
            &merchant_private_key,
            None,
            None,
        )
        .expect("sync error");

    let transaction_id_vec = hex::decode(transaction_id).unwrap();
    let mut transaction_id = [0; 32];
    transaction_id.copy_from_slice(&transaction_id_vec);

    let transaction_id: &TxId = &transaction_id;
    index
        .transaction(transaction_id)
        .expect("transaction error")
}

type AppSigner = DefaultSigner<SledStorage>;
type AppIndex = DefaultIndex<SledStorage, RpcClient>;
type AppTransactionCipher = MockAbciTransactionObfuscation<RpcClient>;
type AppTxBuilder = DefaultTransactionBuilder<AppSigner, LinearFee, AppTransactionCipher>;
type AppWalletClient = DefaultWalletClient<SledStorage, AppIndex, AppTxBuilder>;
type AppTransactionHandler = DefaultTransactionHandler<SledStorage>;
type AppBlockHandler =
    DefaultBlockHandler<AppTransactionCipher, AppTransactionHandler, SledStorage>;
type AppSynchronizer = ManualSynchronizer<SledStorage, RpcClient, AppBlockHandler>;
fn make_app() -> (AppWalletClient, AppIndex, AppSynchronizer) {
    let tendermint_client = RpcClient::new(TENDERMINT_URL);
    let storage = SledStorage::new(".client-storage").unwrap();
    let signer = DefaultSigner::new(storage.clone());
    let transaction_cipher = MockAbciTransactionObfuscation::new(tendermint_client.clone());
    let transaction_handler = DefaultTransactionHandler::new(storage.clone());
    let block_handler = DefaultBlockHandler::new(
        transaction_cipher.clone(),
        transaction_handler,
        storage.clone(),
    );

    let index = DefaultIndex::new(storage.clone(), tendermint_client.clone());
    let transaction_builder = DefaultTransactionBuilder::new(
        signer,
        tendermint_client.genesis().unwrap().fee_policy(),
        transaction_cipher.clone(),
    );
    let wallet = DefaultWalletClient::builder()
        .with_wallet(storage.clone())
        .with_transaction_read(index.clone())
        .with_transaction_write(transaction_builder)
        .build()
        .unwrap();
    let synchronizer =
        ManualSynchronizer::new(storage.clone(), tendermint_client.clone(), block_handler);

    (wallet, index, synchronizer)
}

fn construct_tx(
    wallet_name: String,
    passphrase: SecUtf8,
    wallet: &AppWalletClient,
    record: &Order,
) -> Tx {
    let merchant_address = wallet
        .transfer_addresses(&wallet_name, &passphrase)
        .unwrap()[0]
        .clone();
    let merchant_view_key = wallet.view_key(&wallet_name, &passphrase).unwrap();

    let buyer_address = ExtendedAddr::from_cro(&record.buyer_address[..]).unwrap();

    let transaction_id_vec = hex::decode(&record.payment_transaction_id).unwrap();
    let mut transaction_id = [0; 32];
    transaction_id.copy_from_slice(&transaction_id_vec);

    let inputs = vec![TxoPointer {
        id: transaction_id,
        index: 0,
    }];

    let outputs = match record.status {
        OrderStatus::Delivering => vec![
            TxOut {
                address: merchant_address,
                value: Coin::from_str(&record.amount[..])
                    .unwrap()
                    .sub(Coin::from(10 * 1_0000_0000))
                    .unwrap(),
                valid_from: None,
            },
            TxOut {
                address: buyer_address,
                value: Coin::from(10 * 1_0000_0000),
                valid_from: None,
            },
        ],
        OrderStatus::Refunding => vec![TxOut {
            address: buyer_address,
            value: Coin::from_str(&record.amount[..])
                .unwrap(),
            valid_from: None,
        }],
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

    let network_id = hex::decode(NETWORK_ID).unwrap()[0];
    let attributes = TxAttributes::new_with_access(network_id, access_policies);
    Tx {
        inputs,
        outputs,
        attributes,
    }
}
