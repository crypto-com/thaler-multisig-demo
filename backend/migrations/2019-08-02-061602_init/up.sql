CREATE TABLE order_details(
  order_id TEXT PRIMARY KEY NOT NULL,
  status TEXT NOT NULL,
  buyer_public_key TEXT NOT NULL,
  buyer_view_key TEXT NOT NULL,
  escrow_public_key TEXT NOT NULL,
  escrow_view_key TEXT NOT NULL,
  session_id TEXT NOT NULL,
  payment_transaction_id TEXT NOT NULL,
  settlement_transaction_id TEXT NOT NULL
);
INSERT INTO order_details VALUES
    ('1', 'PendingPayment', '03fc1905a36674d0eb08af473bfac5aa8c24c5177c5aa979e045091e3060dc052c','1','03fc1905a36674d0eb08af473bfac5aa8c24c5177c5aa979e045091e3060dc052c','1','1', '', '');