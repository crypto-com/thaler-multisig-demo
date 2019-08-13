use crate::schema::order_and_session_mapping;
use serde::{Deserialize, Serialize}; 

#[derive(Serialize, Deserialize, Queryable, Insertable)]
#[table_name="order_and_session_mapping"]
pub struct OrderAndSessionMapping {
    pub order_id: String,
    pub session_id: String,
}
#[derive(Deserialize)]
pub struct Order {
    pub order_id: String,
}
#[derive(Serialize)]
pub struct Keys {
    pub pub_key: String,
    pub view_key: String,
}
#[derive(Deserialize)]
pub struct AfterPaid{
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
pub struct AfterReceived{
    pub order_id: String,
    pub signed_txn: String,
    pub nonce: String,
}
#[derive(Serialize)]
pub struct BroadcastedTxn {
    pub tx_id: String,
}