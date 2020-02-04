# Crypto.com Chain Multi-sig backend demo in Actix-Web

Demonstrate Multi-signature support in Crypto.com chain using Customer-Merchant-Escrow example.

## Description

In this setup we demonstrate a 2-of-3 Multi-signature use cases in which a order is created with Customer, Merchant and Escrow. Only Customer and Merchant are involved to complete a 0.5 CRO purchase.

We use the same wallet for the Customer and Escrow as Escrow is not involved so we only need its public key on order creation.

## Parameters

| Parameter | Value |
| --- | --- |
| Purchase | 0.5 CRO |
| Customer Deposit | 10 CRO |
| Transaction fee | 1 CRO |


## Pre-requisite

-  Setup Crypto.com chain Devnet/Tesetnet
    - https://crypto-com.github.io/getting-started/
- Clone latest [Crypto.com chain repository](https://github.com/crypto-com/chain)
- [Postman](https://www.postman.com)
  - Import Postman collections and environments from `./postman` folder

## Steps

### How to follow the steps?

- Steps that involves submitting request at Postman are prefixed `[Postman]`
- Steps that involves updating Postman envionrment are prefixed wih `[Env]`

#### Build and run the server

```bash
# if ubuntu : sudo apt-get install libsqlite3-dev
# if fedora : sudo dnf install libsqlite3x-devel
cargo install diesel_cli --no-default-features --features sqlite
diesel setup
cargo run
# HTTP server started at 127.0.0.1:8080
```

1. #### Create/Restore wallet with more than 15 DCRO/TCRO inside
    - [Env] Update `enckey` to `wallet_enckey`

1. #### Get Customer and Merchant public keys for MultiSig address
    - [Postman] Submit `wallet_newMultiSigAddressPublicKey` or `multiSig_newAddressPublicKey` twice to create two public keys
    - [Env] Update the two public keys to  `public_key` and `escrow_public_key`

1. #### Get Customer and Merchant view key
    - [Postman] Submit `wallet_getViewKey` to retrieve view key
    - [Env] Update the view key to `view_key` and `escrow_view_key`

1. #### Create Customer transfer address for deposit refund later
    - [Postman] Submit `wallet_createTransferAddress` to create a new transfer address
    - [Env] Update the transfer address to `transfer_address`

1. #### Create a new order in Merchant server
    - [Postman] Submit `/order/new`
    - [Env] Copy value in response to corresponding environment
| Response key | Environment key |
| --- | --- |
| `public_key` | `merchan_public_key` |
| `address` | `merchant_address` |
| `view_key` | `merchant_view_key` |
| `multisig_address` | `multisig_address` |

1. #### Customer send funds to the MultiSig address
    - [Postman] Submit `wallet_sendToAddress` to send funds from Customer wallet to the MultiSig address
    - [Env] Update the transaction hash in response to `multisig_utxo_transaction_id`

1. #### Customer submit payment proof to Merchant
    - [Postman] Submit `/order/payment-proof` to submit payment proof
        - This step will take a long time because Merchant server will sync the wallet to verify the transaction
        - In production consider doing background sync of wallet

1. #### Merchant updates that the goods/service has been delivered
    - [Postman] Submit `/order/delivering`
    - [Env] Update the `settlement_transaction_id` in response to `settlement_transaction_id` in environment

1. #### Customer create a new MultiSig session
    - [Postman] Submit `multiSig_newSession`
    - [Env] Update session id in response to `session_id`

1. #### Customer get nonce commitment
    - [Postman] Submit `multiSig_nonceCommitment`
    - Copy the nonce commitment in response to next step

1. #### Customer retrieve nonce commitment and nonce from Merchant
    - [Postman] Submit `/order/exchange-commitment` with body `commitment` set to the nonce commitment obtained from last step
    - Copy the nonce commitment and nonce
    - [Postman] Submit `multiSig_addNonceCommitment` with the nonce commitment
    - [Postman] Submit `multiSig_addNonce` with the nonce

1. #### Customer retrieve nonce and partial signature from MultiSig session
    - [Postman] Submit `multiSig_nonce` to retrieve Customer nonce
    - [Postman] Submit `multiSig_partialSign` to retrieve Customer nonce and nonce commitment

1. #### Customer confirm goods/services delivery submit nonce and partial signature to Merchant
    - [Postman] Submit `/order/confirm/delivery` with the nonce and partial signature from last step
        - On submission, Merchant sever will broadcast the settlement transaction spending the payment in MultiSig address
            - Input: MultiSig address UTXO with 10.5 CRO
            - Output:
                - 0.5 CRO to merchant address
                - 9 CRO deposit refund to customer address
                - 1 CRO transaction fee
            - p.s. 1 CRO transaction fee is for simplicity only. In production a much less CRO fee is required.

## Query order table using sqlite client

```bash
# if ubuntu : sudo apt-get install sqlite3
# if fedora : sudo dnf install sqlite3x
sqlite3 multi-sig.db
sqlite> .tables
sqlite> select * from orders;
```

## Reset the server

```bash
rm -rf .client-storage && diesel migration redo
```

## Watch for changes during development

```bash
systemfd --no-pid -s http::8080 -- cargo watch -x run
```