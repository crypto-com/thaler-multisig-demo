# PoGSD building instructions

## Customer side

- Please follow the [build instructions](https://github.com/crypto-com/multisig-demo#build-prerequisites) to set up the sample wallet and the merchant frontend. You will need to begin with the branch ``multi-sig-demo`` on the [sample wallet.](https://github.com/crypto-com/sample-chain-wallet/tree/multi-sig-demo)
Afterwards, you will find the "Pay" option on the top right corner as follows:
<div>
    <img src="../images/Workflow_1.png" alt="Transaction_Flows_1" />
</div>

- Fill in the *order id* and *amount*; Choose the *Merchant* that you would like to pay and your perferred *Escrow service* provider.
<div>
    <img src="../images/Workflow_2.png" alt="Transaction_Flows_1" />
</div>

- Confirm the detail and press "Send" to proceed. 

<div>
    <img src="../images/Workflow_3.png" alt="Transaction_Flows_1" />
</div>

- Once the transaction was successfully broadcasted, you will be able to see the order in the *Outstanding transactions* list. You can contact the escrow service by clicking *"Involve Escrow"* for resolving a payment dispute.
 
<div>
    <img src="../images/Workflow_4.png" alt="Transaction_Flows_1" />
</div>

## Merchant side 

- On the other hand, merchant can choose to mark the order as *"[Delivered](https://github.com/crypto-com/multisig-demo#scenario-a-the-item-is-shipped)"* or *"[Refund](https://github.com/crypto-com/multisig-demo#b1-reimbursement-without-escrow)"* the customer.
<div>
    <img src="../images/Workflow_5.png" alt="Transaction_Flows_1" />
</div>