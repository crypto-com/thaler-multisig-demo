use crate::schema::multi_sig_utxo;
use crate::schema::partially_signed_transaction;
use serde::{Deserialize, Serialize}; 
#[derive(Serialize, Deserialize, Queryable, Insertable)]
#[table_name="partially_signed_transaction"]
pub struct PartiallySignedTxn {
    pub order_id: String,
    pub tx_id: String,
    pub output_id: i32,
    pub hash: String,
    pub date: String,
}

#[derive(Serialize, Deserialize, Queryable, Insertable)]
#[table_name="multi_sig_utxo"]
pub struct MultiSigUtxo {
    pub order_id: String,
    pub tx_id: String,
    pub output_id: i32,
    pub date: String,
}

#[derive(Serialize)]
pub struct Address {
    pub address: String,
}