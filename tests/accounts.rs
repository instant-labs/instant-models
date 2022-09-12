#![allow(dead_code)]
use postgres::{Config, NoTls};
use sea_query::PostgresQueryBuilder; //

// Example generated with
// `cargo run --bin cli --features="postgres clap" -- -t "accounts" > accounts.rs`
//

pub struct Accounts {
    pub user_id: i32,
    pub username: String,
    pub password: String,
    pub email: String,
    pub created_on: chrono::naive::NaiveDateTime,
    pub last_login: Option<chrono::naive::NaiveDateTime>,
}

pub struct AccountsNew<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub email: &'a str,
    pub created_on: chrono::naive::NaiveDateTime,
    pub last_login: Option<chrono::naive::NaiveDateTime>,
}

// TODO: derive this automatically.
#[derive(sea_query::Iden)]
pub enum AccountsIden {
    Table,
    UserId,
    Username,
    Password,
    Email,
    CreatedOn,
    LastLogin
}

// TODO: derive this automatically
impl AccountsIden {
    pub fn eq(self, value: impl Into<sea_query::Value>) -> sea_query::SimpleExpr {
        sea_query::Expr::col(self).eq(value)
    }

    pub fn is_not_null(self) -> sea_query::SimpleExpr {
        sea_query::Expr::col(self).is_not_null()
    }

    // TODO: re-export rest of sea_query::Expr functions.
}

// TODO: derive this automatically.
impl Accounts {
    pub fn table() -> AccountsIden { AccountsIden::Table }

    // Columns
    pub fn user_id() -> AccountsIden { AccountsIden::UserId }
    pub fn username() -> AccountsIden { AccountsIden::Username }
    pub fn password() -> AccountsIden { AccountsIden::Password }
    pub fn email() -> AccountsIden { AccountsIden::Email }
    pub fn created_on() -> AccountsIden { AccountsIden::CreatedOn }
    pub fn last_login() -> AccountsIden { AccountsIden::LastLogin }

    // Example helper function.
    pub fn all_columns() -> &'static [AccountsIden] {
        &[
            AccountsIden::UserId,
            AccountsIden::Username,
            AccountsIden::Password,
            AccountsIden::Email,
            AccountsIden::CreatedOn,
            AccountsIden::LastLogin,
        ]
    }

    // TODO: export this (and other queries/statements) in a trait instead?
    pub fn filter() -> sea_query::SelectStatement {
        let mut query = sea_query::Query::select();
        // Automatically choose the correct table.
        query.from(AccountsIden::Table);
        query
    }
}

#[test]
fn test_sea_query() {
    let expected = r#"SELECT "user_id", "email", "last_login" FROM "accounts_iden" WHERE "username" = 'foo' AND "last_login" IS NOT NULL LIMIT 1"#;

    let user = "foo";
    let query: String = Accounts::filter()
      .columns([Accounts::user_id(), Accounts::email(), Accounts::last_login()])
      .and_where(Accounts::username().eq(user))
      .and_where(Accounts::last_login().is_not_null())
      .limit(1)
      .to_string(PostgresQueryBuilder);

    assert_eq!(query, expected);
    // let row = sqlx::query!(query)
    //   .fetch_one(&pool)
    //   .await?;

    // TODO: custom trait/s to simplify query/statement execution ergonomics, so they
    //       can be called immediately with, e.g. `.fetch_one(&pool)?`, rather than two steps,
    //       and without the `PostgresQueryBuilder`?

    // TODO: how to use prepared statements to avoid SQL injection?
    //       https://github.com/SeaQL/sea-query/issues/22
}

impl Accounts {
    pub fn insert_slice(
        client: &mut postgres::Client,
        slice: &[AccountsNew<'_>],
    ) -> Result<(), postgres::Error> {
        let statement = client.prepare("INSERT INTO accounts(username, password, email, created_on, last_login) VALUES($1, $2, $3, $4, $5);")?;
        for entry in slice {
            client.execute(
                &statement,
                &[
                    &entry.username,
                    &entry.password,
                    &entry.email,
                    &entry.created_on,
                    &entry.last_login,
                ],
            )?;
        }
        Ok(())
    }
}

#[test]
fn test_accounts() {
    let client = &mut Config::new()
        .user("postgres")
        .password("postgres")
        .host("127.0.0.1")
        .port(5432)
        .dbname("postgres")
        .connect(NoTls)
        .unwrap();

    client
        .batch_execute(
            r#"CREATE TABLE IF NOT EXISTS accounts (
        user_id serial PRIMARY KEY,
        username TEXT UNIQUE NOT NULL,
        password TEXT NOT NULL,
        email TEXT UNIQUE NOT NULL,
        created_on TIMESTAMP NOT NULL,
        last_login TIMESTAMP
);"#,
        )
        .unwrap();
    // tabula rasa - clean slate.
    client.batch_execute(r#"DELETE FROM accounts;"#).unwrap();

    let created_on = chrono::offset::Local::now().naive_local();
    let last_login: Option<chrono::naive::NaiveDateTime> = None;
    let password = "password".to_string();
    let new_val_1 = AccountsNew {
        username: "user1",
        password: &password,
        email: "foo1@example.com",
        created_on,
        last_login,
    };
    let new_val_2 = AccountsNew {
        username: "user2",
        password: &password,
        email: "foo2@example.com",
        created_on,
        last_login,
    };
    let new_val_3 = AccountsNew {
        username: "user3",
        password: &password,
        email: "foo3@example.com",
        created_on,
        last_login,
    };
    let new_val_4 = AccountsNew {
        username: "user4",
        password: &password,
        email: "foo4@example.com",
        created_on,
        last_login,
    };

    Accounts::insert_slice(client, &[new_val_1, new_val_2, new_val_3, new_val_4]).unwrap();

    for row in client
        .query("SELECT user_id, username FROM accounts;", &[])
        .unwrap()
    {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        println!("found person: {} {}", id, name);
    }

    // clean up what we did
    client.batch_execute(r#"DELETE FROM accounts;"#).unwrap();
}
