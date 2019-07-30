Crypto.com Chain Multi-sig backend demo in Actix-Web

## Usage

### init database sqlite

From the root directory of this project:
```bash
bash ./db/setup_db.sh
```

This creates a sqlite database, multi-sig.db, in the root.


### server

```bash
# if ubuntu : sudo apt-get install libsqlite3-dev
# if fedora : sudo dnf install libsqlite3x-devel
cargo run (or ``cargo watch -x run``)
# Started http server: 127.0.0.1:8080
```

### dev with watch
systemfd --no-pid -s http::8080 -- cargo watch -x run

### web client

[http://127.0.0.1:8080/multi-sig-utxo](http://127.0.0.1:8080/multi-sig-utxo)

[http://127.0.0.1:8080/transaction/partially-signed](http://127.0.0.1:8080/transaction/partially-signed)


### sqlite client

```bash
# if ubuntu : sudo apt-get install sqlite3
# if fedora : sudo dnf install sqlite3x
sqlite3 multi-sig.db
sqlite> .tables
sqlite> select * from partially_signed_transaction;
```

