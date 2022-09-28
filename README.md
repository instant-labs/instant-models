# instant-models

## Generate Rust code with the CLI

```shell
cargo run --bin cli --features="postgres clap sql" -- -t "accounts" > accounts.rs
```

```shell
$ cargo run --bin cli --features="postgres clap sql" -- --help
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

## Query Builder

When the `sql` feature is enabled, generated tables include code to compose simple SQL queries.

E.g. Consider the following generated table structure.

```rust
pub struct Accounts {
    pub user_id: i32,
    pub username: String,
    pub password: String,
    pub email: String,
    pub created_on: chrono::naive::NaiveDateTime,
    pub last_login: Option<chrono::naive::NaiveDateTime>,
}
```

We can construct a query to retrieve the username and email address of all users: 
```rust
// SELECT "username", "email" FROM "accounts"
let select: String = Accounts::query()
    .select(|a| [a.username, a.email])
    .to_string();
```

Conditionals can be specified using `filter` and combined using bitwise operators: 
```rust
// SELECT "username", "email" FROM "accounts" 
// WHERE "last_login" IS NOT NULL AND ("user_id" = 1 OR "username" != "admin")
let select: String = Accounts::query()
    .select(|a| [a.username, a.email])
    .filter(|a| Sql::is_not_null(a.last_login) & (Sql::eq(a.user_id, 1) | Sql::ne(a.username, "admin")))
    .to_string();
```

### Fetching Queries

With the `postgres` feature enabled, the query can be excuted directly using the [postgres](https://crates.io/crates/postgres) crate:

```rust
use postgres::{Config, NoTls, Row};

// Connect to a Postgres database.
let client = &mut Config::new()
    .user("postgres")
    .password("postgres")
    .host("127.0.0.1")
    .port(5432)
    .dbname("postgres")
    .connect(NoTls)
    .unwrap();

// Fetch all rows.
let rows: Vec<Row> = Accounts::query()
    .select(|a| [a.username, a.email])
    .fetch(client, &[])
    .unwrap();
```

## Development


### Tests

Start a local, ephemeral Postgres instance:

```shell
docker run -it -p 127.0.0.1:5432:5432 --rm -e POSTGRES_PASSWORD=postgres postgres
```

Run all tests:

```shell
cargo test -- --nocapture
```
