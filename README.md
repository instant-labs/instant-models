# instant-models

Run tests:

```shell
cargo test -- --nocapture
```

## Generate Rust code with the cli


```shell
cargo run --bin cli --features="postgres clap" -- -t "accounts" > accounts.rs
```


```shell
$ cargo run --bin cli --features="postgres clap" -- --help
instant-models 0.1.0
Generate Rust code from postgres table

USAGE:
    cli [OPTIONS] --table-name <TABLE_NAME>

OPTIONS:
    -h, --help                         Print help information
        --pg-dbname <PG_DBNAME>        Postgres db name [default: postgres]
        --pg-host <PG_HOST>            Postgres host [default: 127.0.0.1]
        --pg-password <PG_PASSWORD>    Postgres password [default: postgres]
        --pg-port <PG_PORT>            Postgres port [default: 5432]
        --pg-username <PG_USERNAME>    Postgres username [default: postgres]
    -t, --table-name <TABLE_NAME>      Name of the table to generate
    -V, --version                      Print version information
```
