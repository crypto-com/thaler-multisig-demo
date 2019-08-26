use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::*;
use serde::{Deserialize, Serialize};
use std::io;

use crate::schema::order_details;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "order_details"]
pub struct OrderDetails {
    pub order_id: String,
    pub status: OrderStatus,
    pub buyer_public_key: String,
    pub buyer_view_key: String,
    pub escrow_public_key: String,
    pub escrow_view_key: String,
    pub session_id: String,
    pub payment_transaction_id: String,
    pub settlement_transaction_id: String,
}
#[derive(Debug, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[sql_type = "Text"]
pub enum OrderStatus {
    PendingPayment,
    Paid,
    Delivering,
    Refunding,
    Settled,
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
            OrderStatus::Paid => String::from("Paid"),
            OrderStatus::Delivering => String::from("Delivering"),
            OrderStatus::Refunding => String::from("Refunding"),
            OrderStatus::Settled => String::from("Settled"),
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
            "Paid" => OrderStatus::Paid,
            "Delivering" => OrderStatus::Delivering,
            "Refunding" => OrderStatus::Refunding,
            "Settled" => OrderStatus::Settled,
            _ => return Err("Unsupported order status".into()),
        })
    }
}
#[derive(Deserialize)]
pub struct Order {
    pub order_id: String,
    pub buyer_public_key: String,
    pub buyer_view_key: String,
    pub escrow_public_key: String,
    pub escrow_view_key: String,
}

#[derive(Serialize)]
pub struct Keys {
    pub public_key: String,
    pub view_key: String,
}
#[derive(Deserialize)]
pub struct AfterPaid {
    pub order_id: String,
    pub tx_id: String,
    pub commitment: String,
}
#[derive(Serialize)]
pub struct AfterShipped {
    pub commitment: String,
    pub nonce: String,
}
#[derive(Deserialize)]
pub struct AfterReceived {
    pub order_id: String,
    pub partial_signature: String,
    pub nonce: String,
}
#[derive(Serialize)]
pub struct BroadcastedTxn {
    pub tx_id: String,
}
