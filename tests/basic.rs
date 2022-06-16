use instant_models::*;
use postgres::{Config, NoTls}; // Client

#[test]
fn test_basic() {
    let client = &mut Config::new()
        .user("postgres")
        .password("postgres")
        .host("127.0.0.1")
        .port(5432)
        .dbname("postgres")
        .connect(NoTls)
        .unwrap();

    const TABLE_NAME: &str = "accounts";

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


    let struct_bldr = new_struct_builder(client, TABLE_NAME).unwrap();
    assert_eq!(struct_bldr.columns.len(), 6);
    let result = struct_bldr.build_type();
    println!("final:\n{}", &result);
    assert_eq!(
        result.replace([' ', '\r', '\n'], ""),
        r#"pub struct Accounts {
    user_id: i32,
    username: String,
    password: String,
    email: String,
    created_on: chrono::naive::NaiveDateTime,
    last_login: Option<chrono::naive::NaiveDateTime>,
}"#
        .replace([' ', '\r', '\n'], "")
    );

    println!("new type:\n{}", struct_bldr.build_new_type());
}
