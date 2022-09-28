use instant_models::{Sql, Table};
use postgres::{Config, NoTls};

// Example generated with
// `cargo run --bin cli --features="postgres clap" -- -t "accounts" > accounts.rs`

pub struct Accounts {
    pub user_id: i32,
    pub created_on: chrono::naive::NaiveDateTime,
    pub last_login: Option<chrono::naive::NaiveDateTime>,
    pub username: String,
    pub password: String,
    pub email: String,
}

pub struct AccountsNew<'a> {
    pub created_on: chrono::naive::NaiveDateTime,
    pub last_login: Option<chrono::naive::NaiveDateTime>,
    pub username: &'a str,
    pub password: &'a str,
    pub email: &'a str,
}

impl Accounts {
    pub fn insert_slice(
        client: &mut postgres::Client,
        slice: &[AccountsNew<'_>],
    ) -> Result<(), postgres::Error> {
        let statement = client.prepare("INSERT INTO accounts(created_on, last_login, username, password, email) VALUES($1, $2, $3, $4, $5);")?;
        for entry in slice {
            client.execute(
                &statement,
                &[
                    &entry.created_on,
                    &entry.last_login,
                    &entry.username,
                    &entry.password,
                    &entry.email,
                ],
            )?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub enum AccountsIden {
    Table,
    UserId,
    CreatedOn,
    LastLogin,
    Username,
    Password,
    Email,
}

impl sea_query::Iden for AccountsIden {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "accounts",
                Self::UserId => "user_id",
                Self::CreatedOn => "created_on",
                Self::LastLogin => "last_login",
                Self::Username => "username",
                Self::Password => "password",
                Self::Email => "email",
            }
        )
        .expect("AccountsIden failed to write");
    }
}

pub struct AccountsFields {
    pub user_id: ::instant_models::Field<i32, AccountsIden>,
    pub created_on: ::instant_models::Field<chrono::naive::NaiveDateTime, AccountsIden>,
    pub last_login: ::instant_models::Field<Option<chrono::naive::NaiveDateTime>, AccountsIden>,
    pub username: ::instant_models::Field<String, AccountsIden>,
    pub password: ::instant_models::Field<String, AccountsIden>,
    pub email: ::instant_models::Field<String, AccountsIden>,
}

impl instant_models::Table for Accounts {
    type FieldsType = AccountsFields;
    const FIELDS: Self::FieldsType = AccountsFields {
        user_id: ::instant_models::Field::new("user_id", AccountsIden::UserId),
        created_on: ::instant_models::Field::new("created_on", AccountsIden::CreatedOn),
        last_login: ::instant_models::Field::new("last_login", AccountsIden::LastLogin),
        username: ::instant_models::Field::new("username", AccountsIden::Username),
        password: ::instant_models::Field::new("password", AccountsIden::Password),
        email: ::instant_models::Field::new("email", AccountsIden::Email),
    };

    fn table() -> sea_query::TableRef {
        use sea_query::IntoTableRef;
        AccountsIden::Table.into_table_ref()
    }
}

#[test]
fn test_accounts_insert() {
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

    let select_query = Accounts::query().select(|a| (a.user_id, a.username));
    assert_eq!(
        select_query.to_string(),
        r#"SELECT "user_id", "username" FROM "accounts""#
    );

    for row in select_query.fetch(client, &[]).unwrap() {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        println!("found person: {} {}", id, name);
    }

    // clean up what we did
    client.batch_execute(r#"DELETE FROM accounts;"#).unwrap();
}

#[test]
fn test_accounts_query() {
    // SELECT single column.
    let select = Accounts::query().select(|a| (a.user_id,)).to_string();
    assert_eq!(select, r#"SELECT "user_id" FROM "accounts""#);

    // SELECT multiple columns.
    let select_multiple = Accounts::query()
        .select(|a| (a.user_id, a.username, a.email))
        .to_string();
    assert_eq!(
        select_multiple,
        r#"SELECT "user_id", "username", "email" FROM "accounts""#
    );

    // SELECT WHERE single condition.
    let select_where = Accounts::query()
        .select(|a| (a.user_id,))
        .filter(|a| Sql::is_null(a.last_login))
        .to_string();
    assert_eq!(
        select_where,
        r#"SELECT "user_id" FROM "accounts" WHERE "last_login" IS NULL"#
    );

    // SELECT WHERE AND.
    let select_where_and = Accounts::query()
        .select(|a| (a.user_id,))
        .filter(|a| Sql::is_null(a.last_login) & Sql::is_not_null(a.created_on))
        .to_string();
    assert_eq!(
        select_where_and,
        r#"SELECT "user_id" FROM "accounts" WHERE "last_login" IS NULL AND "created_on" IS NOT NULL"#
    );

    // SELECT WHERE OR.
    let select_where_or = Accounts::query()
        .select(|a| (a.user_id,))
        .filter(|a| Sql::is_null(a.last_login) | Sql::is_not_null(a.created_on))
        .to_string();
    assert_eq!(
        select_where_or,
        r#"SELECT "user_id" FROM "accounts" WHERE "last_login" IS NULL OR "created_on" IS NOT NULL"#
    );

    // SELECT WHERE AND OR.
    let select_where_and_or = Accounts::query()
        .select(|a| (a.user_id,))
        .filter(|a| {
            Sql::is_null(a.last_login) & (Sql::is_not_null(a.created_on) | Sql::eq(a.user_id, 1))
        })
        .to_string();
    assert_eq!(
        select_where_and_or,
        r#"SELECT "user_id" FROM "accounts" WHERE "last_login" IS NULL AND ("created_on" IS NOT NULL OR "user_id" = 1)"#
    );
}
