use basictest::*;
use postgres::{Config, NoTls}; //

#[test]
fn test_test() {
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
    client.batch_execute(r#"DELETE FROM accounts;
ALTER SEQUENCE accounts_user_id_seq RESTART;"#).unwrap();

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

    let _where = Where::gt(Accounts::USER_ID, 1_i32);
    let q = Query::new(
        false,
        vec![
            "user_id".into(),
            "username".into(),
            "password".into(),
            "email".into(),
        ],
        "accounts".into(),
        Some(_where),
    );
    println!("query to string: {}", q.to_string());

    for row in client.query(&q.to_string(), &[]).unwrap() {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        println!("found person: {} {}", id, name);
    }

    let _where1 = Where::gt(Accounts::USER_ID, 1_i32);
    let _where = Where::and(_where1, Where::eq(Accounts::USERNAME, "user4".to_string()));
    let q = Query::new(
        false,
        vec![
            "user_id".into(),
            "username".into(),
            "password".into(),
            "email".into(),
        ],
        "accounts".into(),
        Some(_where),
    );
    println!("query to string: {}", q.to_string());

    for row in client.query(&q.to_string(), &[]).unwrap() {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        println!("found person: {} {}", id, name);
    }

    // clean up what we did
    client.batch_execute(r#"DELETE FROM accounts;"#).unwrap();

}
