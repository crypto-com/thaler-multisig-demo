table! {
    order_details (order_id) {
        order_id -> Text,
        buyer_public_key -> Text,
        buyer_view_key -> Text,
        escrow_public_key -> Text,
        escrow_view_key -> Text,
        session_id -> Nullable<Text>,
    }
}
