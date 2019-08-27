function makeRandomString(length) {
  var result = "";
  var characters =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  var charactersLength = characters.length;
  for (var i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }
  return result;
}

function makeRandomAmount() {
  return Math.floor(Math.random() * 10000).toString();
}

function makeRecord(size) {
  let type = "";
  let amount = "";
  let buyer_public_key = "";
  let buyer_view_key = "";
  let buyer_address = "";
  let escrow_public_key = "";
  let escrow_view_key = "";
  let session_id = "";
  let payment_transaction_id = "";
  let settlement_transaction_id = "";
  for (let i = 0; i < size; i += 1) {
    const statusId = Math.floor(Math.random() * 6);
    buyer_public_key = makeRandomString(64);
    amount = makeRandomAmount();
    buyer_view_key = makeRandomString(64);
    buyer_address = `dcro${makeRandomString(24)}`;
    escrow_public_key = makeRandomString(64);
    escrow_view_key = makeRandomString(64);
    switch (statusId) {
      case 0:
        type = "PendingPayment";
        break;
      case 1:
        type = "PendingResponse";
        payment_transaction_id = makeRandomString(64);
        break;
      case 2:
        type = "Delivering";
        payment_transaction_id = makeRandomString(64);
        session_id = makeRandomString(64);
        break;
      case 3:
        type = "Refunding";
        payment_transaction_id = makeRandomString(64);
        session_id = makeRandomString(64);
        break;
      case 4:
        type = "Completed";
        payment_transaction_id = makeRandomString(64);
        settlement_transaction_id = makeRandomString(64);
        session_id = makeRandomString(64);
        break;
      case 5:
        type = "Refunded";
        payment_transaction_id = makeRandomString(64);
        settlement_transaction_id = makeRandomString(64);
        session_id = makeRandomString(64);
        break;
    }
    console.log(
      `INSERT INTO order_details VALUES('${i}','${type}','${amount}','${buyer_public_key}','${buyer_view_key}','${buyer_address}','${escrow_public_key}','${escrow_view_key}','${session_id}','${payment_transaction_id}','${settlement_transaction_id}');`
    );
  }
}

makeRecord(50);
