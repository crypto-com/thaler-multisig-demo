/*
   Crypto.com Chain Multi-sig backend demo in Actix-Web
*/
#[macro_use]
extern crate diesel;

use actix_cors::Cors;
use actix_web::{http::header, middleware, web, App, Error as AWError, HttpResponse, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use futures::future::{self, Future};
use listenfd::ListenFd;
use secstr::SecUtf8;
use std::ops::Add;
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
use client_common::{PublicKey, Transaction, Storage};
use client_core::signer::DefaultSigner;
use client_core::transaction_builder::{DefaultTransactionBuilder, UnauthorizedTransactionBuilder};
use client_core::wallet::{DefaultWalletClient, MultiSigWalletClient, WalletClient};
use client_core::TransactionBuilder;
use client_index::cipher::MockAbciTransactionObfuscation;
use client_index::handler::{DefaultBlockHandler, DefaultTransactionHandler};
use client_index::index::{DefaultIndex, Index, UnauthorizedIndex};
use client_index::synchronizer::ManualSynchronizer;

use crate::models::*;

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

mod db;
mod models;
mod schema;

const NETWORK_ID: &str = "AB";
const TENDERMINT_URL: &str = "http://localhost:16657";

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

fn new_order(
    pool: web::Data<Pool>,
    params: web::Form<NewOrderRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    let (wallet, _, _) = make_app();
    let wallet_name = Uuid::new_v4().to_string();
    let passphrase = SecUtf8::from("passphrase");

    wallet.new_wallet(&wallet_name, &passphrase).expect("new_wallet error");

    let merchant_address = wallet.new_transfer_address(&wallet_name, &passphrase).unwrap();
    let merchant_public_key = &wallet.public_keys(&wallet_name, &passphrase).unwrap()[0];
    let merchant_public_key = merchant_public_key.to_owned();
    let merchant_view_key = wallet.view_key(&wallet_name, &passphrase).unwrap();

    let buyer_public_key = PublicKey::from_str(&params.buyer_public_key.to_string()).unwrap();
    let escrow_public_key = PublicKey::from_str(&params.escrow_public_key.to_string()).unwrap();

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

    let res = NewOrderResponse {
        public_key: merchant_public_key.to_string(),
        address: merchant_address.to_string(),
        view_key: merchant_view_key.to_string(),
        multisig_address: multisig_address.to_string(),
    };

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
    db::execute_register_order(pool, order)
        .from_err()
        .and_then(|_| Ok(HttpResponse::Ok().json(res)))
}

fn submit_payment_proof(
    pool: web::Data<Pool>,
    params: web::Form<PaymentProof>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    let order_id = params.order_id.to_string();
    let transaction_id = params.transaction_id.to_string();
    let transaction = get_transaction_by_id(transaction_id, order_id.clone())
        .expect("get_transaction_by_id error");

    let storage = SledStorage::new(".client-storage").unwrap();
    let wallet = DefaultWalletClient::builder()
        .with_wallet(storage.clone())
        .build()
        .unwrap();
    let passphrase = SecUtf8::from("passphrase");
    let name = params.order_id.to_string();

    db::execute_get_order_by_id(pool.clone(), order_id.clone())
        .from_err()
        .and_then(move |record| {
            if let Transaction::TransferTransaction(tx) = transaction {
                Ok((tx, record))
            } else {
                Err(AWError::from(
                    HttpResponse::BadRequest()
                        .reason("Invalid transaction type")
                        .finish(),
                ))
            }
        })
        .and_then(move |(tx, record)| {
            if tx.outputs.len() == 0 {
                return Err(AWError::from(
                    HttpResponse::BadRequest()
                        .reason("Transaction has no output")
                        .finish(),
                ));
            }

            let merchant_public_key = &wallet.public_keys(&name, &passphrase).unwrap()[0];
            let merchant_public_key = merchant_public_key.to_owned();
            let buyer_public_key =
                PublicKey::from_str(&record.buyer_public_key.to_string()).unwrap();
            let escrow_public_key =
                PublicKey::from_str(&record.escrow_public_key.to_string()).unwrap();
            let multisig_address = wallet
                .new_multisig_transfer_address(
                    &name,
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
                        .reason("Incorrect transaction output address")
                        .finish(),
                ));
            }
            if tx.outputs[0].value != Coin::from_str(&record.amount).unwrap() {
                return Err(AWError::from(
                    HttpResponse::BadRequest()
                        .reason("Incorrect transaction output amount")
                        .finish(),
                ));
            }
            Ok(())
        })
        .and_then(move |_| {
            db::execute_store_payment_transaction_id(
                pool,
                params.order_id.to_string(),
                params.transaction_id.to_string(),
            )
        })
        .and_then(|_| {
            let res = OrderUpdatedResponse { order_id };
            Ok(HttpResponse::Ok().json(res))
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
    params: web::Form<ExchangeCommitmentRequest>,
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

    db::execute_get_order_by_id(pool.clone(), name.clone())
        .from_err()
        .and_then(move |record| {
            // TODO: Should check order hasn't exchanged commitment before
            let merchant_public_key = wallet.public_keys(&name, &passphrase).unwrap()[0].clone();
            let merchant_address =
                wallet.transfer_addresses(&name, &passphrase).unwrap()[0].clone();
            let merchant_view_key = wallet.view_key(&name, &passphrase).unwrap();

            let buyer_public_key =
                PublicKey::from_str(&record.buyer_public_key.to_string()).unwrap();
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
                        value: Coin::from_str(&record.amount[..]).unwrap(),
                        valid_from: None,
                    },
                    TxOut {
                        address: buyer_address,
                        // FIXME: Change back after debug
                        value: Coin::from(10),
                        valid_from: None,
                    },
                ],
                OrderStatus::Refunding => vec![TxOut {
                    address: buyer_address,
                    value: Coin::from_str(&record.amount[..])
                        .unwrap()
                        .add(Coin::from(10 * 10_000_000))
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
                order_id: params.order_id.to_string(),
                commitment: hex::encode(merchant_nonce_commitment),
                nonce: merchant_nonce.to_string(),
                transaction_id: hex::encode(transaction.id()),
                transaction: transaction.clone(),
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
    params: web::Form<ConfirmRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    // TODO: Should check order is in delivering status
    confirm(pool, params)
}
fn confirm_refund(
    pool: web::Data<Pool>,
    params: web::Form<ConfirmRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    // TODO: Should check order is in refunding status
    confirm(pool, params)
}
fn confirm(
    pool: web::Data<Pool>,
    params: web::Form<ConfirmRequest>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    let buyer_partial_signature_vec = hex::decode(params.partial_signature.to_string()).unwrap();
    let mut buyer_partial_signature = [0; 32];
    buyer_partial_signature.copy_from_slice(&buyer_partial_signature_vec);

    let buyer_nonce = PublicKey::from_str(&params.nonce.to_string()).unwrap();

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
        .with_transaction_read(index)
        .with_transaction_write(transaction_builder)
        .build()
        .unwrap();
    let synchronizer =
        ManualSynchronizer::new(storage.clone(), tendermint_client.clone(), block_handler);

    let passphrase = SecUtf8::from("passphrase");
    let name = params.order_id.to_string();

    db::execute_get_order_by_id(pool.clone(), name.clone())
        .from_err()
        .and_then(move |record| {
            // TODO: Should check if user is confirming the same status as DB record

            // Complete multi-sig session
            let session_id_vec = hex::decode(record.session_id.to_string()).unwrap();
            let mut session_id = [0; 32];
            session_id.copy_from_slice(&session_id_vec);
            let buyer_public_key =
                PublicKey::from_str(&record.buyer_public_key.to_string()).unwrap();

            wallet
                .add_nonce(&session_id, &passphrase, &buyer_nonce, &buyer_public_key)
                .expect("add_nonce error");

            wallet
                .partial_signature(&session_id, &passphrase)
                .expect("partial_signature error");

            wallet
                .add_partial_signature(
                    &session_id,
                    &passphrase,
                    buyer_partial_signature,
                    &buyer_public_key,
                )
                .expect("add_partial_signature error");

            let signature = wallet
                .signature(&session_id, &passphrase)
                .expect("signature error");

            // Construct transaction for signing and broadcast
            let merchant_public_key = wallet.public_keys(&name, &passphrase).unwrap()[0].clone();
            let merchant_address =
                wallet.transfer_addresses(&name, &passphrase).unwrap()[0].clone();
            let merchant_view_key = wallet.view_key(&name, &passphrase).unwrap();

            let buyer_public_key =
                PublicKey::from_str(&record.buyer_public_key.to_string()).unwrap();
            let buyer_address = ExtendedAddr::from_cro(&record.buyer_address[..]).unwrap();

            let transaction_id_vec = hex::decode(&record.payment_transaction_id).unwrap();
            let mut transaction_id = [0; 32];

            // DEBUG:
            use chain_core::tx::witness::TxInWitness;
            use chain_tx_validation::witness::verify_tx_address;
            let escrow_public_key =
                PublicKey::from_str(&record.escrow_public_key.to_string()).unwrap();
            let multisig_address = wallet
                .new_multisig_transfer_address(
                    &name,
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
            let proof = wallet
                .generate_proof(
                    &name,
                    &passphrase,
                    &multisig_address,
                    vec![merchant_public_key.clone(), buyer_public_key.clone()],
                )
                .unwrap();
            let witness = TxInWitness::TreeSig(signature, proof);
            assert!(verify_tx_address(&witness, &transaction_id, &multisig_address).is_ok());
            // DEBUG:

            let merchant_private_key = wallet
                .private_key(&passphrase, &merchant_view_key)
                .unwrap()
                .unwrap();
            let merchant_staking_addresses = wallet.staking_addresses(&name, &passphrase).unwrap();
            synchronizer.sync(
                &merchant_staking_addresses,
                &merchant_view_key,
                &merchant_private_key,
                None,
                None,
            );

            let inputs = vec![TxoPointer {
                id: transaction_id,
                index: 0,
            }];

            let outputs = match record.status {
                OrderStatus::Delivering => vec![
                    TxOut {
                        address: merchant_address,
                        value: Coin::from_str(&record.amount[..]).unwrap(),
                        valid_from: None,
                    },
                    TxOut {
                        address: buyer_address,
                        value: Coin::from(10 * 10_000_000),
                        valid_from: None,
                    },
                ],
                OrderStatus::Refunding => vec![TxOut {
                    address: buyer_address,
                    value: Coin::from_str(&record.amount[..])
                        .unwrap()
                        .add(Coin::from(10 * 10_000_000))
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

            let network_id = hex::decode("AB").unwrap()[0];
            let attributes = TxAttributes::new_with_access(network_id, access_policies);
            let transaction = Tx {
                inputs,
                outputs,
                attributes,
            };

            let tx_aux = wallet
                .transaction(&name, &session_id, &passphrase, transaction)
                .expect("transaction error");

            wallet
                .broadcast_transaction(&tx_aux)
                .expect("broadcast_transaction error");

            let res = ConfirmResponse {
                transaction_id: record.settlement_transaction_id.to_string(),
            };
            Ok(HttpResponse::Ok().json(res))
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

fn get_transaction_by_id(transaction_id: String, order_id: String) -> Option<Transaction> {
    let tendermint_client = RpcClient::new(TENDERMINT_URL);
    let storage = SledStorage::new(".client-storage").unwrap();
    let index = DefaultIndex::new(storage.clone(), tendermint_client.clone());
    let transaction_cipher = MockAbciTransactionObfuscation::new(tendermint_client.clone());
    let transaction_handler = DefaultTransactionHandler::new(storage.clone());
    let block_handler = DefaultBlockHandler::new(
        transaction_cipher.clone(),
        transaction_handler,
        storage.clone(),
    );

    let wallet = DefaultWalletClient::builder()
        .with_wallet(storage.clone())
        .build()
        .unwrap();
    let synchronizer =
        ManualSynchronizer::new(storage.clone(), tendermint_client.clone(), block_handler);

    let passphrase = SecUtf8::from("passphrase");
    let name = order_id.to_string();

    let merchant_view_key = wallet.view_key(&name, &passphrase).unwrap();
    let merchant_private_key = wallet
        .private_key(&passphrase, &merchant_view_key)
        .unwrap()
        .unwrap();
    let merchant_staking_addresses = wallet.staking_addresses(&name, &passphrase).unwrap();
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
        server.bind("127.0.0.1:8080").unwrap()
    };

    server.run().unwrap();
}
