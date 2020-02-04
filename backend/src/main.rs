/*
   Crypto.com Chain Multi-sig backend demo in Actix-Web
*/
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;

use std::env;

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
use chain_core::init::network::{Network, MAINNET_CHAIN_ID, TESTNET_CHAIN_ID};
use chain_core::tx::data::access::{TxAccess, TxAccessPolicy};
use chain_core::tx::data::address::ExtendedAddr;
use chain_core::tx::data::attribute::TxAttributes;
use chain_core::tx::data::input::TxoPointer;
use chain_core::tx::data::output::TxOut;
use chain_core::tx::data::{Tx, TxId};
use chain_core::tx::fee::LinearFee;
use chain_core::tx::TransactionId;
use client_common::storage::SledStorage;
use client_common::tendermint::types::GenesisExt;
use client_common::tendermint::{Client, WebsocketRpcClient};
use client_common::PublicKey;
use client_core::cipher::DefaultTransactionObfuscation;
use client_core::service::WalletStateService;
use client_core::signer::WalletSignerManager;
use client_core::transaction_builder::DefaultWalletTransactionBuilder;
use client_core::types::AddressType;
use client_core::types::{TransactionChange, WalletKind};
use client_core::wallet::syncer::{ObfuscationSyncerConfig, TxObfuscationDecryptor, WalletSyncer};
use client_core::wallet::{DefaultWalletClient, MultiSigWalletClient, WalletClient};

use crate::models::*;

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

mod db;
mod models;
mod schema;

lazy_static! {
    static ref CHAIN_ID: String = env::var("CHAIN_ID")
        .unwrap_or_else(|_| "testnet-thaler-crypto-com-chain-42".to_string());
    static ref NETWORK_ID: u8 = {
        let chain_id = CHAIN_ID.clone();
        let length = chain_id.len();
        let hexstring = &chain_id[(length - 2)..];
        hex::decode(hexstring).expect("last two characters should be hex digits")[0]
    };
    static ref NETWORK: Network = {
        let chain_id = CHAIN_ID.clone();
        match chain_id.as_str() {
            MAINNET_CHAIN_ID => Network::Mainnet,
            TESTNET_CHAIN_ID => Network::Testnet,
            _ => Network::Devnet,
        }
    };
    static ref TENDERMINT_WEBSOCKET_URL: String = env::var("TENDERMINT_WEBSOCKET_URL")
        .unwrap_or_else(|_| "ws://127.0.0.1:26657/websocket".to_string());
    static ref TQE_ADDRESS: Option<String> = env::var("TQE_ADDRESS")
        .map(|address| Some(address))
        .unwrap_or_else(|_| None);
    static ref CUSTOMER_DEPOSIT: Coin = Coin::from(10 * 1_0000_0000);
    static ref FIXED_FEE: Coin = Coin::one();
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
                .new_wallet(&wallet_name, &passphrase, WalletKind::Basic)
                .expect("Unable to create new wallet");

            let merchant_address = wallet
                .new_public_key(&wallet_name, &passphrase, Some(AddressType::Transfer))
                .expect("Unable to create public key");
            let merchant_public_keys = &wallet.public_keys(&wallet_name, &passphrase).unwrap();
            let merchant_public_key = merchant_public_keys
                .iter()
                .next()
                .expect("Wallet has no public key");
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
                multisig_address: multisig_address.to_cro(*NETWORK).unwrap(),
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

            Ok((transaction, record))
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

            let merchant_public_keys: Vec<PublicKey> = wallet.public_keys(&wallet_name, &passphrase)
                .map(|keys| keys.into_iter().collect())
                .expect("Unable to get public keys");
            if merchant_public_keys.is_empty() {
                panic!("Wallet has no public key");
            }
            let merchant_public_key = &merchant_public_keys[0];

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
                )
                .unwrap();
            if tx.outputs[0].address.to_cro(*NETWORK).unwrap() != multisig_address.to_cro(*NETWORK).unwrap() {
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
            let res = OrderPaidResponse {
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

    let (wallet, _, _) = make_app();
    let passphrase = SecUtf8::from("passphrase");

    let new_status = status.clone();

    db::execute_is_order_exist(query_pool.clone(), query_order_id.clone())
        .and_then(|exist| {
            if !exist {
                return Err(AWError::from(HttpResponse::NotFound().finish()));
            }
            Ok(())
        })
        .and_then(move |_| {
            // TODO: Check status
            db::execute_update_order_status(update_pool, update_order_id.clone(), status)
        })
        .and_then(move |_| db::execute_get_order_by_id(query_pool, query_order_id))
        .and_then(move |mut record| {
            let wallet_name = record.wallet_name.clone();
            record.status = new_status;
            let transaction =
                construct_tx(wallet_name.clone(), passphrase.clone(), &wallet, &record);

            let res = OrderUpdatedResponse {
                order_id: return_order_id,
                status: record.status,
                settlement_transaction_id: hex::encode(transaction.id()),
                settlement_transaction: transaction,
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

            let merchant_public_keys: Vec<PublicKey> = wallet.public_keys(&wallet_name, &passphrase)
                .map(|keys| keys.into_iter().collect())
                .expect("Unable to get public keys");
            if merchant_public_keys.is_empty() {
                panic!("Wallet has no public key");
            }
            let merchant_public_key = &merchant_public_keys[0];

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

    let (wallet, _, syncer_config) = make_app();
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

            let transaction_id_vec = hex::decode(&record.payment_transaction_id).unwrap();
            let mut transaction_id = [0; 32];
            transaction_id.copy_from_slice(&transaction_id_vec);

            // TODO: Create separate thread to sync in background
            let syncer = make_syncer(&wallet_name, &passphrase, syncer_config);
            syncer.sync().expect("sync error");

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

fn get_transaction_by_id(transaction_id: String, wallet_name: String) -> Option<TransactionChange> {
    let (_, wallet_state_service, syncer_config) = make_app();
    let passphrase = SecUtf8::from("passphrase");

    // TODO: Create separate thread to sync in background
    let syncer = make_syncer(&wallet_name, &passphrase, syncer_config);
    syncer.sync().expect("sync error");

    let transaction_id_vec = hex::decode(transaction_id).unwrap();
    let mut transaction_id = [0; 32];
    transaction_id.copy_from_slice(&transaction_id_vec);

    let transaction_id: &TxId = &transaction_id;
    wallet_state_service
        .get_transaction_change(&wallet_name, &passphrase, transaction_id)
        .expect("get_transaction_change error")
}

type AppTransactionCipher = DefaultTransactionObfuscation;
type AppWalletStateService = WalletStateService<SledStorage>;
type AppTxBuilder = DefaultWalletTransactionBuilder<SledStorage, LinearFee, AppTransactionCipher>;
type AppWalletClient = DefaultWalletClient<SledStorage, WebsocketRpcClient, AppTxBuilder>;
type AppSyncerConfig =
    ObfuscationSyncerConfig<SledStorage, WebsocketRpcClient, AppTransactionCipher>;
type AppTxObfuscationDecryptor = TxObfuscationDecryptor<AppTransactionCipher>;
type AppSyncer = WalletSyncer<SledStorage, WebsocketRpcClient, AppTxObfuscationDecryptor>;

fn make_app() -> (AppWalletClient, AppWalletStateService, AppSyncerConfig) {
    let tendermint_client = WebsocketRpcClient::new(&TENDERMINT_WEBSOCKET_URL)
        .expect("Unable to create WebsocketRpcClient");
    let storage = SledStorage::new(".client-storage").unwrap();
    let transaction_cipher = if TQE_ADDRESS.is_some() {
        let address = TQE_ADDRESS.as_ref().unwrap();
        if let Some(hostname) = address.split(':').next() {
            DefaultTransactionObfuscation::new(
                address.to_string(),
                hostname.to_string(),
            )
        } else {
            panic!("Unable to decode TQE_ADDRESS");
        }
    } else {
        DefaultTransactionObfuscation::from_tx_query(&tendermint_client)
            .expect("Unable to create DefaultTransactionObfuscation")
    };

    let signer_manager = WalletSignerManager::new(storage.clone());
    let transaction_builder = DefaultWalletTransactionBuilder::new(
        signer_manager,
        tendermint_client.genesis().unwrap().fee_policy(),
        transaction_cipher.clone(),
    );
    let wallet_client = DefaultWalletClient::new(
        storage.clone(),
        tendermint_client.clone(),
        transaction_builder,
    );

    let wallet_state_service = WalletStateService::new(storage.clone());

    let enable_fast_forward = true;
    let batch_size = 20;
    let syncer_config = AppSyncerConfig::new(
        storage.clone(),
        tendermint_client.clone(),
        transaction_cipher.clone(),
        enable_fast_forward,
        batch_size,
    );

    (wallet_client, wallet_state_service, syncer_config)
}

fn make_syncer(
    wallet_name: &str,
    wallet_passphrase: &SecUtf8,
    syncer_config: AppSyncerConfig,
) -> AppSyncer {
    WalletSyncer::with_obfuscation_config(
        syncer_config,
        None,
        wallet_name.to_owned(),
        wallet_passphrase.to_owned(),
    )
    .expect("Unable to create WalletSyncer")
}

fn construct_tx(
    wallet_name: String,
    passphrase: SecUtf8,
    wallet: &AppWalletClient,
    record: &Order,
) -> Tx {
    let merchant_address = wallet
        .transfer_addresses(&wallet_name, &passphrase)
        .unwrap()
        .iter()
        .next()
        .expect("Wallet has not transfer address")
        .clone();
    let merchant_view_key = wallet.view_key(&wallet_name, &passphrase).unwrap();

    let buyer_address = ExtendedAddr::from_cro(&record.buyer_address[..], *NETWORK).unwrap();

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
                    .sub(*CUSTOMER_DEPOSIT)
                    .unwrap(),
                valid_from: None,
            },
            TxOut {
                address: buyer_address,
                value: CUSTOMER_DEPOSIT.sub(*FIXED_FEE).unwrap(),
                valid_from: None,
            },
        ],
        OrderStatus::Refunding => vec![TxOut {
            address: buyer_address,
            value: Coin::from_str(&record.amount[..]).unwrap(),
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

    let attributes = TxAttributes::new_with_access(*NETWORK_ID, access_policies);
    Tx {
        inputs,
        outputs,
        attributes,
    }
}
