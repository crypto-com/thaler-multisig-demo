Crypto.com Chain Multi-sig backend demo in Actix-Web

## Usage

### server

```bash
# if ubuntu : sudo apt-get install libsqlite3-dev
# if fedora : sudo dnf install libsqlite3x-devel
cargo install diesel_cli --no-default-features --features sqlite
diesel setup
cargo run (or ``cargo watch -x run``)
# Started http server: 127.0.0.1:8080
```

### to reset everything

rm -rf .client-storage && diesel migration redo

### dev with watch

systemfd --no-pid -s http::8080 -- cargo watch -x run

### sqlite client

```bash
# if ubuntu : sudo apt-get install sqlite3
# if fedora : sudo dnf install sqlite3x
sqlite3 multi-sig.db
sqlite> .tables
sqlite> select * from partially_signed_transaction;
```
