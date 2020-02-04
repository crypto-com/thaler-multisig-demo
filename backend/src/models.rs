use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::*;
use serde::{Deserialize, Serialize};
use std::io;

use chain_core::tx::data::Tx;

use crate::schema::orders;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "orders"]
pub struct Order {
    pub order_id: String,
    pub status: OrderStatus,
    pub wallet_name: String,
    pub amount: String,
    pub buyer_public_key: String,
    pub buyer_view_key: String,
    pub buyer_address: String,
    pub escrow_public_key: String,
    pub escrow_view_key: String,
    pub session_id: String,
    pub payment_transaction_id: String,
    pub settlement_transaction_id: String,
}
#[derive(Debug, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Clone, Copy)]
#[sql_type = "Text"]
pub enum OrderStatus {
    PendingPayment,
    PendingResponse,
    Delivering,
    Refunding,
    Completed,
    Refunded,
}
impl<DB: Backend> ToSql<Text, DB> for OrderStatus
where
    String: ToSql<Text, DB>,
{
    fn to_sql<W>(&self, out: &mut Output<W, DB>) -> serialize::Result
    where
        W: io::Write,
    {
        let v = match *self {
            OrderStatus::PendingPayment => String::from("PendingPayment"),
            OrderStatus::PendingResponse => String::from("PendingResponse"),
            OrderStatus::Delivering => String::from("Delivering"),
            OrderStatus::Refunding => String::from("Refunding"),
            OrderStatus::Completed => String::from("Completed"),
            OrderStatus::Refunded => String::from("Refunded"),
        };
        v.to_sql(out)
    }
}
impl<DB: Backend> FromSql<Text, DB> for OrderStatus
where
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        let v = String::from_sql(bytes)?;
        Ok(match &v[..] {
            "PendingPayment" => OrderStatus::PendingPayment,
            "PendingResponse" => OrderStatus::PendingResponse,
            "Delivering" => OrderStatus::Delivering,
            "Refunding" => OrderStatus::Refunding,
            "Completed" => OrderStatus::Completed,
            "Refunded" => OrderStatus::Refunded,
            _ => return Err("Unsupported order status".into()),
        })
    }
}
#[derive(Deserialize)]
pub struct NewOrderRequest {
    pub order_id: String,
    pub amount: String,
    pub buyer_public_key: String,
    pub buyer_view_key: String,
    pub buyer_address: String,
    pub escrow_public_key: String,
    pub escrow_view_key: String,
}
#[derive(Serialize)]
pub struct NewOrderResponse {
    pub public_key: String,
    pub address: String,
    pub view_key: String,
    pub multisig_address: String,
}
#[derive(Deserialize)]
pub struct PaymentProof {
    pub order_id: String,
    pub transaction_id: String,
}
#[derive(Deserialize)]
pub struct OrderRequest {
    pub order_id: String,
}
#[derive(Serialize)]
pub struct OrderPaidResponse {
    pub order_id: String,
}
#[derive(Serialize)]
pub struct OrderUpdatedResponse {
    pub order_id: String,
    pub status: OrderStatus,
    pub settlement_transaction_id: String,
    pub settlement_transaction: Tx,
}
#[derive(Serialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub amount: String,
    pub status: OrderStatus,
    pub buyer_public_key: String,
    pub buyer_view_key: String,
    pub buyer_address: String,
    pub escrow_public_key: String,
    pub escrow_view_key: String,
    pub session_id: String,
    pub payment_transaction_id: String,
    pub settlement_transaction_id: String,
    // pub nonce_commitment: String,
    // pub nonce: String,
}
#[derive(Deserialize)]
pub struct ExchangeCommitmentRequest {
    pub order_id: String,
    pub commitment: String,
}
#[derive(Serialize)]
pub struct ExchangeCommitmentResponse {
    pub order_id: String,
    pub commitment: String,
    pub nonce: String,
    pub transaction_id: String,
    pub transaction: Tx,
}
#[derive(Deserialize)]
pub struct ConfirmRequest {
    pub order_id: String,
    pub nonce: String,
    pub partial_signature: String,
}
#[derive(Serialize)]
pub struct ConfirmResponse {
    pub order_id: String,
    pub transaction_id: String,
}
#[derive(Deserialize)]
pub struct AfterReceived {
    pub order_id: String,
    pub partial_signature: String,
    pub nonce: String,
}
