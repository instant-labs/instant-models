use instant_models::{Field, Sql, Table};
use sea_query::TableRef;
use std::fmt::Write;

// Example manually constructed.

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

pub struct Access {
    pub user: i32,
    pub domain: i32,
    pub role: String,
}

#[derive(Copy, Clone)]
pub enum AccessIden {
    Table,
    User,
    Domain,
    Role,
}

impl sea_query::Iden for AccessIden {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "access",
                Self::User => "user",
                Self::Domain => "domain",
                Self::Role => "role",
            }
        )
        .expect("AccessIden failed to write");
    }
}

pub struct AccessFields {
    pub table: AccessIden,
    pub user: Field<i32, AccessIden>,
    pub domain: Field<i32, AccessIden>,
    pub role: Field<String, AccessIden>,
}

impl Table for Access {
    type FieldsType = AccessFields;
    const FIELDS: Self::FieldsType = AccessFields {
        table: AccessIden::Table,
        user: Field::new("user", AccessIden::User),
        domain: Field::new("domain", AccessIden::Domain),
        role: Field::new("role", AccessIden::Role),
    };

    fn table() -> TableRef {
        use sea_query::IntoTableRef;
        AccessIden::Table.into_table_ref()
    }
}

pub struct Examples {
    pub id: i32,
    pub example: String,
}

#[derive(Copy, Clone)]
pub enum ExamplesIden {
    Table,
    Id,
    Example,
}

pub struct ExamplesFields {
    pub table: ExamplesIden,
    pub id: Field<i32, ExamplesIden>,
    pub example: Field<String, ExamplesIden>,
}

impl sea_query::Iden for ExamplesIden {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "examples",
                Self::Id => "id",
                Self::Example => "example",
            }
        )
        .expect("ExampleIden failed to write");
    }
}

impl Table for Examples {
    type FieldsType = ExamplesFields;
    const FIELDS: Self::FieldsType = ExamplesFields {
        table: ExamplesIden::Table,
        id: Field::new("id", ExamplesIden::Id),
        example: Field::new("example", ExamplesIden::Example),
    };

    fn table() -> TableRef {
        use sea_query::IntoTableRef;
        ExamplesIden::Table.into_table_ref()
    }
}

#[test]
fn test_query_join() {
    let expected = r#"SELECT "user_id", "username", "password", "email"
FROM "accounts", "access", "examples"
WHERE "username" = 'foo'
AND ("last_login" IS NOT NULL OR "created_on" <> '1970-01-01 00:00:00')
AND ("user_id" = "access"."user" AND "role" = 'DomainAdmin')
AND "user_id" = "examples"."id"
LIMIT 1"#;

    let user = "foo";
    let role = "DomainAdmin";
    let timestamp = chrono::NaiveDateTime::from_timestamp(0, 0);

    let query = Accounts::query()
        .filter(|a| {
            Sql::eq(a.username, user)
                & (Sql::is_not_null(a.last_login) | Sql::ne(a.created_on, timestamp))
        })
        .from::<Access>()
        .filter(|(a, acl)| Sql::equals(a.user_id, acl.table, acl.user) & Sql::eq(acl.role, role))
        .from::<Examples>()
        .filter(|(a, .., ex)| Sql::equals(a.user_id, ex.table, ex.id))
        .select(|(a, ..)| (a.user_id, a.username, a.password, a.email))
        .limit(1)
        .to_string();

    assert_eq!(query, expected.replace('\n', " "));
}
