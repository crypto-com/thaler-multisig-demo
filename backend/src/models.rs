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
#[sql_type = "SmallInt"]
pub enum OrderStatus {
    PendingPayment,
    Paid,
    Delivering,
    Refunding,
    Settled,
}
impl<DB: Backend> ToSql<SmallInt, DB> for OrderStatus
where
    i16: ToSql<SmallInt, DB>,
{
    fn to_sql<W>(&self, out: &mut Output<W, DB>) -> serialize::Result
    where
        W: io::Write,
    {
        let v = match *self {
            OrderStatus::PendingPayment => 1,
            OrderStatus::Paid => 2,
            OrderStatus::Delivering => 3,
            OrderStatus::Refunding => 4,
            OrderStatus::Settled => 5,
        };
        v.to_sql(out)
    }
}
impl<DB: Backend> FromSql<SmallInt, DB> for OrderStatus
where
    i16: FromSql<SmallInt, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        let v = i16::from_sql(bytes)?;
        Ok(match v {
            1 => OrderStatus::PendingPayment,
            2 => OrderStatus::Paid,
            3 => OrderStatus::Delivering,
            4 => OrderStatus::Refunding,
            5 => OrderStatus::Settled,
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
