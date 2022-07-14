use instant_models::StructBuilder;
use postgres::{Config, NoTls}; // Client
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::process::Command;

// Uncomment to prevent automatic cleanup of generated project
//const TMP_DIR: Option<&'static str> = Some("/tmp/basic/");
const TMP_DIR: Option<&'static str> = None;

const RUN_CARGO_TESTS: bool = true;

/// Create a library crate and make sure the generated code type checks.
fn create_cargo_project(builder: StructBuilder, cargo_test: bool) -> Result<(), anyhow::Error> {
    println!("creating cargo project...");
    let cargo = Command::new("cargo")
        .arg("init")
        .arg("--lib")
        .arg("--name")
        .arg("basictest")
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()
        .expect("failed to execute process");
    if !cargo.status.success() {
        return Err(anyhow::anyhow!("cargo init returned error"));
    }
    let mut file = File::create("./src/lib.rs")?;

    let header = r#"#![allow(dead_code)]

"#;
    file.write_all(header.as_bytes())?;
    file.write_all(include_bytes!("../src/select.rs"))?;
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
    if cargo_test {
        println!("Creating test file at ./tests/basic.rs...");
        std::fs::create_dir("./tests")?;
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open("./tests/basic.rs")?;

        file.write_all(include_bytes!("./basic/testfile.rs"))?;
    } else {
        println!("Not creating test file.");
    }

    println!("running `cargo check`...");
    let check = Command::new("cargo")
        .arg("check")
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()
        .expect("failed to execute process");
    if !check.status.success() {
        return Err(anyhow::anyhow!("cargo check returned error"));
    }

    if cargo_test {
        println!("running `cargo test`...");
        let check = Command::new("cargo")
            .arg("test")
            .arg("--")
            .arg("--nocapture")
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()
            .expect("failed to execute process");
        if !check.status.success() {
            return Err(anyhow::anyhow!("cargo test returned error"));
        }
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
    let tmpdir = if let Some(tmpdir) = TMP_DIR {
        let _ = std::fs::create_dir(tmpdir);
        std::env::set_current_dir(tmpdir).unwrap();
        println!("Generating project at {}", tmpdir);
        None
    } else {
        let tmpdir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(tmpdir.path()).unwrap();
        println!("Generating project at {}", tmpdir.path().display());
        Some(tmpdir)
    };
    let ret = create_cargo_project(struct_bldr, RUN_CARGO_TESTS);
    std::env::set_current_dir(&cwd).unwrap();
    if let Some(d) = tmpdir {
        d.close().unwrap();
    }
    ret.unwrap();
}
