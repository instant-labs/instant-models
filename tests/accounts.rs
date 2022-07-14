#![allow(dead_code)]
use postgres::{Config, NoTls}; //

include!("../src/select.rs");

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

impl Accounts {
    pub const TABLE_NAME: &'static str = "accounts";
    pub const USER_ID: Column<i32> = Column {
        table: Self::TABLE_NAME,
        name: "user_id",
        phantom: std::marker::PhantomData,
    };
    pub const USERNAME: Column<String> = Column {
        table: Self::TABLE_NAME,
        name: "username",
        phantom: std::marker::PhantomData,
    };
    pub const PASSWORD: Column<String> = Column {
        table: Self::TABLE_NAME,
        name: "password",
        phantom: std::marker::PhantomData,
    };
    pub const EMAIL: Column<String> = Column {
        table: Self::TABLE_NAME,
        name: "email",
        phantom: std::marker::PhantomData,
    };
    pub const CREATED_ON: Column<chrono::naive::NaiveDateTime> = Column {
        table: Self::TABLE_NAME,
        name: "created_on",
        phantom: std::marker::PhantomData,
    };
    pub const LAST_LOGIN: Column<chrono::naive::NaiveDateTime> = Column {
        table: Self::TABLE_NAME,
        name: "last_login",
        phantom: std::marker::PhantomData,
    };
    pub const ALL_COLUMNS: &'static [&'static str] = &[
        "user_id",
        "username",
        "password",
        "email",
        "created_on",
        "last_login",
    ];

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

    pub fn all(client: &mut postgres::Client) -> Result<Vec<Self>, postgres::Error> {
        let q = Query::new(
            false,
            Self::ALL_COLUMNS
                .iter()
                .map(|&c| c.into())
                .collect::<Vec<_>>(),
            Self::TABLE_NAME.into(),
            Query::<bool>::NONE,
        );

        let mut ret = vec![];
        for row in client.query(&q.to_string(), &[]).unwrap() {
            ret.push(Self::try_from(row)?);
        }
        Ok(ret)
    }
}

impl TryFrom<postgres::row::Row> for Accounts {
    type Error = postgres::error::Error;
    fn try_from(row: postgres::row::Row) -> Result<Self, Self::Error> {
        Ok(Self {
            user_id: row.try_get("user_id")?,
            username: row.try_get("username")?,
            password: row.try_get("password")?,
            email: row.try_get("email")?,
            created_on: row.try_get("created_on")?,
            last_login: row.try_get("last_login")?,
        })
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
    client
        .batch_execute(
            r#"DELETE FROM accounts;
ALTER SEQUENCE accounts_user_id_seq RESTART;"#,
        )
        .unwrap();

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

    let all = Accounts::all(client).unwrap();
    assert_eq!(all.len(), 4);
    for acc in all {
        assert!(acc.user_id < 6);
        if acc.user_id == 1 {
            assert_eq!(acc.username, "user1");
            assert_eq!(acc.password, password);
            assert_eq!(acc.email, "foo1@example.com");
        }
    }

    // clean up what we did
    client.batch_execute(r#"DELETE FROM accounts;"#).unwrap();
}

#[test]
fn test_expr() {
    let a = Where::lt(Accounts::USER_ID, 5_i32);
    assert_eq!(format!("{}", a.to_sql()), "(accounts.user_id) < (5)");
    let b = Where::eq(
        Where::column("accounts".into(), "username".into()),
        Either::expr("hello-world".to_string()),
    );
    let c = Where::eq(Accounts::USERNAME, "hello-world".to_string());
    assert_eq!(b.to_sql(), c.to_sql());
    assert_eq!(
        format!("{}", b.to_sql()),
        "(accounts.username) = ($accounts$hello-world$accounts$)"
    );
}
