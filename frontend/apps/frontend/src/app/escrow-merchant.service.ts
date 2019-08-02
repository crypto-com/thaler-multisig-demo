import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, forkJoin } from 'rxjs'
import { map } from 'rxjs/operators';

@Injectable({
  providedIn: 'root'
})
export class EscrowMerchantService {
  private baseUrl = 'http://localhost:8080';

  constructor(private http: HttpClient) { }

  getOrderList(): Observable<OrderRequest[]> {
    return forkJoin([
      this.getMultiSigUtxo(),
      this.getPartiallySignedTransaction(),
    ]).pipe(
      map((result) => {
        const multiSigUtxoList = result[0];
        const partiallySignedTransactionList = result[1];

        return multiSigUtxoList.map(multiSigUtxo => {
          const order: OrderRequest = {
            order_id: multiSigUtxo.order_id,
            tx_id: multiSigUtxo.tx_id,
            output_id: multiSigUtxo.output_id,
            status: OrderStatus.PENDING,
            requested_at: multiSigUtxo.date,
          };
          const partiallySignedTransaction = partiallySignedTransactionList.find(tx => {
            return tx.order_id === multiSigUtxo.order_id &&
              tx.tx_id === multiSigUtxo.tx_id &&
              tx.output_id === multiSigUtxo.output_id
          });

          if (partiallySignedTransaction) {
            order.status = OrderStatus.MULTISIG_IN_PROGRESS;
          }

          return order;
        });
      }),
    );
  }

  getMultiSigUtxo(): Observable<MultiSigUtxo[]> {
    const url = `${this.baseUrl}/multi-sig-utxo`;
    return this.http.get<MultiSigUtxo[]>(url);
  }

  getPartiallySignedTransaction(): Observable<PartiallySignedTransaction[]> {
    const url = `${this.baseUrl}/transaction/partially-signed`;
    return this.http.get<PartiallySignedTransaction[]>(url);
  }
}


export type OrderRequest = OrderBase & {
  status: OrderStatus;
  requested_at: string;
}

enum OrderStatus {
  PENDING,
  DELIVERED,
  MULTISIG_IN_PROGRESS,
  PAID,
  REFUNDED,
}

type MultiSigUtxo = OrderBase & {
  date: string;
}

type PartiallySignedTransaction = OrderBase & {
  hash: string;
  date: string;
}

type OrderBase = {
  order_id: string;
  tx_id: string;
  output_id: number;
}
