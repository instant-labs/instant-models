#![allow(dead_code)]

use postgres::{Config, NoTls};
use sea_query::{ConditionalStatement, JoinType, PostgresQueryBuilder, Query};
use std::fmt::Write;
use std::marker::PhantomData;
use instant_models::{Sql, Table};

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

pub struct Access {
    pub user: i32,
    pub domain: i32,
    pub role: String,
}

#[derive(sea_query::Iden, Copy, Clone)]
pub enum AccessIden {
    Table,
    User,
    Domain,
    Role,
}

pub struct AccessFields {
    pub table: AccessIden,
    pub user: AccessIden,
    pub domain: AccessIden,
    pub role: AccessIden,
}

pub const ACCESS_FIELDS: AccessFields = AccessFields {
    table: AccessIden::Table,
    user: AccessIden::User,
    domain: AccessIden::Domain,
    role: AccessIden::Role,
};

pub struct Example {
    pub id: i32,
    pub example: String,
}

#[derive(sea_query::Iden, Copy, Clone)]
pub enum ExampleIden {
    Table,
    Id,
    Example,
}

pub struct ExampleFields {
    pub table: ExampleIden,
    pub id: ExampleIden,
    pub example: ExampleIden,
}

pub const EXAMPLE_FIELDS: ExampleFields = ExampleFields {
    table: ExampleIden::Table,
    id: ExampleIden::Id,
    example: ExampleIden::Example,
};

// TODO: derive this automatically.
#[derive(Copy, Clone)]
pub enum AccountsIden {
    Table,
    UserId,
    Username,
    Password,
    Email,
    CreatedOn,
    LastLogin,
}

impl sea_query::Iden for AccountsIden {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "accounts",
                Self::UserId => "user_id",
                Self::Username => "username",
                Self::Password => "password",
                Self::Email => "email",
                Self::CreatedOn => "created_on",
                Self::LastLogin => "last_login",
            }
        )
        .expect("AccountsIden failed to write");
    }
}

impl Table for Accounts {
    type Fields = AccountsFields;
}

impl Table for Access {
    type Fields = AccessFields;
}

impl Table for Example {
    type Fields = ExampleFields;
}

pub trait IdenFields: sea_query::Iden {
    type Fields;

    fn table() -> Self;
    fn fields() -> Self::Fields;
}

impl IdenFields for AccountsIden {
    type Fields = AccountsFields;

    fn table() -> Self {
        AccountsIden::Table
    }

    fn fields() -> Self::Fields {
        ACCOUNTS_FIELDS
    }
}

impl IdenFields for AccessIden {
    type Fields = AccessFields;

    fn table() -> Self {
        AccessIden::Table
    }

    fn fields() -> Self::Fields {
        ACCESS_FIELDS
    }
}

impl IdenFields for ExampleIden {
    type Fields = ExampleFields;

    fn table() -> Self {
        ExampleIden::Table
    }

    fn fields() -> Self::Fields {
        EXAMPLE_FIELDS
    }
}



pub struct AccountsFields {
    pub user_id: AccountsIden,
    pub username: AccountsIden,
    pub password: AccountsIden,
    pub email: AccountsIden,
    pub created_on: AccountsIden,
    pub last_login: AccountsIden,
}

pub const ACCOUNTS_FIELDS: AccountsFields = AccountsFields {
    user_id: AccountsIden::UserId,
    username: AccountsIden::Username,
    password: AccountsIden::Password,
    email: AccountsIden::Email,
    created_on: AccountsIden::CreatedOn,
    last_login: AccountsIden::LastLogin,
};

// // TODO: derive this automatically
// impl AccountsIden {
//     pub fn eq(self, value: impl Into<sea_query::Value>) -> Sql {
//         Sql::eq(self, value)
//     }
//
//     pub fn ne(self, value: impl Into<sea_query::Value>) -> Sql {
//         Sql::ne(self, value)
//     }
//
//     pub fn is_not_null(self) -> Sql {
//         Sql::is_not_null(self)
//     }
//
// }


#[test]
fn test_sea_query() {
    let expected = r#"SELECT "user_id", "username", "password", "email"
FROM "accounts", "access", "example"
WHERE "username" = 'foo'
    AND ("last_login" IS NOT NULL OR "created_on" <> '1970-01-01 00:00:00')
    AND ("user_id" = "access"."user" AND "role" = 'DomainAdmin')
    AND "user_id" = "example"."id"
"#;

    let user = "foo";
    let role = "DomainAdmin";
    let timestamp = chrono::NaiveDateTime::from_timestamp(0, 0);

    let query = Accounts::query()
        .select(|a| [a.user_id, a.username, a.password, a.email])
        .where_(|a| {
            Sql::eq(a.username, user)
                & (Sql::is_not_null(a.last_login) | a.created_on.ne(timestamp))
        })
        .from::<Access>()
        .where_(|(a, acl)| Sql::equals(a.user_id, acl.table, acl.user) & Sql::eq(acl.role, role))
        .join::<Example>()
        .where_(|(a, .., ex)| Sql::equals(a.user_id, ex.table, ex.id))
        .to_string();

    assert_eq!(query, expected.replace('\n', " "));
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
        let statement = client.prepare(
            "INSERT INTO accounts(username, password, email, created_on, last_login) VALUES($1, $2, $3, $4, $5);",
        )?;
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
