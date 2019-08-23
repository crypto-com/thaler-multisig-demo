use crate::schema::order_details;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "order_details"]
pub struct OrderDetails {
    pub order_id: String,
    pub status: 
    pub buyer_public_key: String,
    pub buyer_view_key: String,
    pub escrow_public_key: String,
    pub escrow_view_key: String,
    pub session_id: String,
    pub payment_transaction_id: String,
    pub settlement_transaction_id: String,
}
#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
pub enum OrderStatus {
    registered,
    pending,
    settled,
    refunded,
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
