#![allow(dead_code)]

use std::fmt::Write;
use std::marker::PhantomData;
use postgres::{Config, NoTls};
use sea_query::{ConditionalStatement, PostgresQueryBuilder, Query}; //

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

pub const FAKE_FIELDS: ExampleFields = ExampleFields {
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
    LastLogin
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
        ).expect("AccountsIden failed to write");
    }
}

pub trait Table {
    type Iden: IdenFields + 'static;
}

impl Table for Accounts {
    type Iden = AccountsIden;
}

impl Table for Access {
    type Iden = AccessIden;
}

impl Table for Example {
    type Iden = ExampleIden;
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
        FAKE_FIELDS
    }
}

pub trait Sources {
    type SOURCES;

    fn sources() -> Self::SOURCES;
    fn tables() -> Vec<sea_query::TableRef>;
    fn as_sources(self) -> Self;
}

pub trait Combine<O> {
    type COMBINED;

    fn combine(other: O) -> Self::COMBINED;
}

impl<T: IdenFields + 'static> Sources for T {
    type SOURCES = T::Fields;

    fn sources() -> Self::SOURCES {
        T::fields()
    }

    fn tables() -> Vec<sea_query::TableRef> {
        use sea_query::IntoTableRef;
        vec![T::table().into_table_ref()]
    }

    fn as_sources(self) -> Self {
        self
    }
}

impl<T: IdenFields + 'static, O: IdenFields + 'static> Combine<O> for T {
    type COMBINED = (T, O);

    fn combine(other: O) -> Self::COMBINED {
        (T::table(), other)
    }
}

impl<A: IdenFields + 'static, B: IdenFields + 'static> Sources for (A, B) {
    type SOURCES = (A::Fields, B::Fields);

    fn sources() -> Self::SOURCES {
        (A::fields(), B::fields())
    }

    fn tables() -> Vec<sea_query::TableRef> {
        use sea_query::IntoTableRef;
        vec![A::table().into_table_ref(), B::table().into_table_ref()]
    }

    fn as_sources(self) -> Self {
        self
    }
}

impl<A: IdenFields + 'static, B: IdenFields + 'static, O: IdenFields + 'static> Combine<O> for (A, B) {
    type COMBINED = (A, B, O);

    fn combine(other: O) -> Self::COMBINED {
        (A::table(), B::table(), other)
    }
}

macro_rules! impl_sources_tuple {
    ( $( $name:ident )+ ) => {
        impl<$($name: IdenFields + 'static),+> Sources for ($($name,)+)
        {
            type SOURCES = ($($name::Fields,)+);

            fn sources() -> Self::SOURCES {
                ($($name::fields(),)+)
            }

            fn tables() -> Vec<sea_query::TableRef> {
                use sea_query::IntoTableRef;
                vec![$($name::table().into_table_ref(),)+]
            }

            fn as_sources(self) -> Self {
                self
            }
        }
    };
    ( $( $name:ident )+, $joinable:expr ) => {
        impl_sources_tuple!($($name)+);

        impl<Z: IdenFields + 'static, $($name: IdenFields + 'static),+> Combine<Z> for ($($name,)+) {
            type COMBINED = ($($name,)+ Z);

            fn combine(other: Z) -> Self::COMBINED {
                ($($name::table(),)+ other).as_sources()
            }
        }
    };
}

impl_sources_tuple! { A, true }
// impl_sources_tuple! { A B, true }
impl_sources_tuple! { A B C, true }
impl_sources_tuple! { A B C D, true }
impl_sources_tuple! { A B C D E, true }
impl_sources_tuple! { A B C D E F, true }
impl_sources_tuple! { A B C D E F G, true }
impl_sources_tuple! { A B C D E F G H, true }
impl_sources_tuple! { A B C D E F G H I, true }
impl_sources_tuple! { A B C D E F G H I J, true }
impl_sources_tuple! { A B C D E F G H I J K, true }
impl_sources_tuple! { A B C D E F G H I J K L } // does not implement Join.

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
    last_login: AccountsIden::LastLogin
};

pub struct Sql {
    cond: sea_query::Cond,
}

impl Sql {
    pub fn eq<T, V>(col: T, value: V) -> Self
    where T: sea_query::IntoColumnRef, V: Into<sea_query::Value> {
        Self {
            cond: sea_query::Cond::all().add(sea_query::Expr::col(col).eq(value))
        }
    }

    pub fn equals<T, U, V>(left: T, table: U, right: V) -> Self
    where T: sea_query::IntoColumnRef, U: sea_query::IntoIden, V: sea_query::IntoIden {
        Self {
            cond: sea_query::Cond::all().add(sea_query::Expr::col(left).equals(table, right))
        }
    }

    pub fn ne<T, V>(col: T, value: V) -> Self
        where T: sea_query::IntoColumnRef, V: Into<sea_query::Value> {
        Self {
            cond: sea_query::Cond::all().add(sea_query::Expr::col(col).ne(value))
        }
    }

    pub fn is_not_null<T>(col: T) -> Self
    where T: sea_query::IntoColumnRef {
        Self {
            cond: sea_query::Cond::all().add(sea_query::Expr::col(col).is_not_null())
        }
    }
}

impl std::ops::BitAnd for Sql {
    type Output = Sql;

    fn bitand(self, rhs: Self) -> Self::Output {
        Sql {
            cond: sea_query::Cond::all().add(self.cond).add(rhs.cond)
        }
    }
}

impl std::ops::BitOr for Sql {
    type Output = Sql;

    fn bitor(self, rhs: Self) -> Self::Output {
        Sql {
            cond: sea_query::Cond::any().add(self.cond).add(rhs.cond)
        }
    }
}

// TODO: derive this automatically
impl AccountsIden {
    pub fn eq(self, value: impl Into<sea_query::Value>) -> Sql {
        Sql::eq(self, value)
    }

    pub fn ne(self, value: impl Into<sea_query::Value>) -> Sql {
        Sql::ne(self, value)
    }

    pub fn is_not_null(self) -> Sql {
        Sql::is_not_null(self)
    }

    // TODO: port rest of sea_query::Expr functions.
}

pub struct SqlQuery<T> {
    sources: PhantomData<T>,
    // TODO: replace SelectStatement with something custom.
    query: sea_query::SelectStatement,
}

// TODO: replace sea_query::Iden with something custom.
impl<T: Sources> SqlQuery<T> {
    pub fn new() -> SqlQuery<T> {
        let mut query = sea_query::SelectStatement::new();
        for table in T::tables() {
            query.from(table);
        }
        Self {
            query,
            sources: PhantomData::<T>,
        }
    }

    pub fn select<F, C, I>(mut self, columns: F) -> Self
    where
    F: FnOnce(T::SOURCES) -> I,
    C: sea_query::IntoColumnRef,
    I: IntoIterator<Item = C>,
    {
        self.query.columns(columns(T::sources()));
        self
    }

    pub fn where_<F>(mut self, conditions: F) -> Self
    where F: FnOnce(T::SOURCES) -> Sql {
        self.query.cond_where(conditions(T::sources()).cond);
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.query.limit(limit);
        self
    }

    pub fn to_string(&self) -> String {
        self.query.to_string(PostgresQueryBuilder)
    }
}

impl<T: Sources> SqlQuery<T> {
    fn from<O: Table>(mut self) -> SqlQuery<T::COMBINED>
    where T: Combine<O::Iden> {
        self.query.from(O::Iden::table());
        SqlQuery {
            sources: PhantomData::<T::COMBINED>,
            query: self.query
        }
    }
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
    pub fn query() -> SqlQuery<AccountsIden> {
        SqlQuery::new()
    }
}

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
      .where_(|a| Sql::eq(a.username, user) & (Sql::is_not_null(a.last_login) | a.created_on.ne(timestamp)))
      .from::<Access>()
      .where_(|(a, acl)| Sql::equals(a.user_id, acl.table, acl.user) & Sql::eq(acl.role, role))
      .from::<Example>()
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
