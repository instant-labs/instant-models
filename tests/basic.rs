use instant_models::StructBuilder;
use postgres::{Config, NoTls}; // Client
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::process::Command;

/// Create a library crate and make sure the generated code type checks.
fn create_cargo_project(builder: StructBuilder) -> Result<(), anyhow::Error> {
    let cargo = Command::new("cargo")
        .arg("init")
        .arg("--lib")
        .arg("--name")
        .arg("basictest")
        //.stdout(Stdio::inherit())
        //.stderr(Stdio::inherit())
        .output()
        .expect("failed to execute process");
    if !cargo.status.success() {
        return Err(anyhow::anyhow!("cargo init returned error"));
    }
    let mut file = File::create("./src/lib.rs")?;

    let header = r#"#![allow(dead_code)]

"#;
    file.write_all(header.as_bytes())?;
    file.write_all(builder.build_type().as_bytes())?;
    file.write_all(builder.build_new_type().as_bytes())?;
    file.write_all(builder.build_type_methods().as_bytes())?;
    drop(file);
    let mut manifest_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("./Cargo.toml")?;

    manifest_file.write_all(
        r#"
chrono = "0.4"
postgres = { version = "0.19.3", features = ["with-chrono-0_4", ] }
"#
        .as_bytes(),
    )?;
    drop(manifest_file);
    let check = Command::new("cargo")
        .arg("check")
        //.stdout(Stdio::inherit())
        //.stderr(Stdio::inherit())
        .output()
        .expect("failed to execute process");
    if !check.status.success() {
        return Err(anyhow::anyhow!("cargo check returned error"));
    }

    Ok(())
}

#[test]
#[ignore]
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

    let struct_bldr = StructBuilder::new_from_conn(client, TABLE_NAME).unwrap();
    assert_eq!(struct_bldr.columns.len(), 6);
    let result = struct_bldr.build_type();
    println!("final:\n{}", &result);
    assert_eq!(
        result.replace([' ', '\r', '\n'], ""),
        r#"pub struct Accounts {
    pub user_id: i32,
    pub username: String,
    pub password: String,
    pub email: String,
    pub created_on: chrono::naive::NaiveDateTime,
    pub last_login: Option<chrono::naive::NaiveDateTime>,
}"#
        .replace([' ', '\r', '\n'], "")
    );

    let cwd = std::env::current_dir().unwrap();
    let tmpdir = tempfile::tempdir().unwrap();
    std::env::set_current_dir(tmpdir.path()).unwrap();
    let ret = create_cargo_project(struct_bldr);
    std::env::set_current_dir(&cwd).unwrap();
    tmpdir.close().unwrap();
    ret.unwrap();
}
