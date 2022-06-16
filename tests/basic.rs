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

    let mut struct_bldr = StructBuilder::new(TABLE_NAME.into());
    for row in client.query("SELECT column_name, is_nullable, data_type FROM information_schema.columns WHERE table_name = $1;", &[&TABLE_NAME]).unwrap() {
        let column_name: &str = row.get(0);
        let is_nullable: &str = row.get(1);
        let data_type: &str = row.get(2);
        let col = Column::new(column_name.to_string().into(), Type::from_str(data_type).unwrap()).set_null(is_nullable == "YES");
        struct_bldr.add_column(col);
    }

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
