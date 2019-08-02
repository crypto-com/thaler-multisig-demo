table! {
    multi_sig_utxo (order_id) {
        order_id -> Text,
        tx_id -> Text,
        output_id -> Integer,
        date -> Text,
    }
}

table! {
    partially_signed_transaction (order_id) {
        order_id -> Text,
        tx_id -> Text,
        output_id -> Integer,
        hash -> Text,
        date -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    multi_sig_utxo,
    partially_signed_transaction,
);
