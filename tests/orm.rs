use chrono;
use postgres::{Config, Error, NoTls}; // Client
use std::marker::PhantomData;

#[derive(Debug)]
pub struct NewInstance;

#[derive(Debug)]
pub struct Modified;

#[derive(Debug)]
pub struct DbValue;

#[derive(Debug)]
pub struct Accounts<State> {
    user_id: i32,
    username: String,
    password: String,
    email: String,
    created_on: chrono::naive::NaiveDateTime,
    last_login: Option<chrono::naive::NaiveDateTime>,
    _state_marker: PhantomData<*const State>,
}

impl Accounts<()> {
    pub fn get(
        user_id: i32,
        connection: &mut postgres::Client,
    ) -> Result<Accounts<DbValue>, Error> {
        let row =
            connection.query_one("SELECT * FROM accounts WHERE user_id = $1;", &[&user_id])?;
        Ok(Accounts {
            user_id: row.get("user_id"),
            username: row.get("username"),
            password: row.get("password"),
            email: row.get("email"),
            created_on: row.get("created_on"),
            last_login: row.get("last_login"),
            _state_marker: PhantomData,
        })
    }

    pub fn all(connection: &mut postgres::Client) -> Result<Vec<Accounts<DbValue>>, Error> {
        let rows = connection.query("SELECT * FROM accounts;", &[])?;
        Ok(rows
            .into_iter()
            .map(|row| Accounts {
                user_id: row.get("user_id"),
                username: row.get("username"),
                password: row.get("password"),
                email: row.get("email"),
                created_on: row.get("created_on"),
                last_login: row.get("last_login"),
                _state_marker: PhantomData,
            })
            .collect::<Vec<_>>())
    }
}

impl Accounts<NewInstance> {
    pub fn new() -> Self {
        Self {
            user_id: 0,
            username: String::new(),
            password: String::new(),
            email: String::new(),
            created_on: chrono::offset::Local::now().naive_local(),
            last_login: None,
            _state_marker: PhantomData,
        }
    }

    pub fn with_username(self, username: String) -> Self {
        Self { username, ..self }
    }

    pub fn with_password(self, password: String) -> Self {
        Self { password, ..self }
    }

    pub fn with_email(self, email: String) -> Self {
        Self { email, ..self }
    }

    pub fn save(self, connection: &mut postgres::Client) -> Result<Accounts<DbValue>, Error> {
        let row = connection.query_one("INSERT INTO accounts (username, password, email, created_on, last_login) VALUES ($1, $2, $3, $4, $5) RETURNING user_id;", &[&self.username, &self.password, &self.email, &self.created_on, &self.last_login])?;
        let user_id: i32 = row.get(0);
        let Self {
            username,
            password,
            email,
            created_on,
            last_login,
            ..
        } = self;
        Ok(Accounts {
            user_id,
            username,
            password,
            email,
            created_on,
            last_login,
            _state_marker: PhantomData,
        })
    }
}

impl Accounts<DbValue> {
    pub fn delete(self, connection: &mut postgres::Client) -> Result<(), Error> {
        connection.execute("DELETE FROM accounts WHERE user_id = $1", &[&self.user_id])?;
        Ok(())
    }

    pub fn edit(self) -> Accounts<Modified> {
        let Self {
            user_id,
            username,
            password,
            email,
            created_on,
            last_login,
            ..
        } = self;
        Accounts {
            user_id,
            username,
            password,
            email,
            created_on,
            last_login,
            _state_marker: PhantomData,
        }
    }
}

impl Accounts<Modified> {
    pub fn set_username(&mut self, username: String) -> &mut Self {
        self.username = username;
        self
    }

    pub fn set_password(&mut self, password: String) -> &mut Self {
        self.password = password;
        self
    }

    pub fn set_email(&mut self, email: String) -> &mut Self {
        self.email = email;
        self
    }

    /* set_created_on, set_last_login, etc... */

    pub fn save(self, connection: &mut postgres::Client) -> Result<Accounts<DbValue>, Error> {
        connection.execute("UPDATE accounts SET username = $1,  password = $2, email = $3, created_on = $4, last_login = $5 WHERE user_id = $6;", &[&self.username, &self.password, &self.email, &self.created_on, &self.last_login, &self.user_id])?;
        let Self {
            user_id,
            username,
            password,
            email,
            created_on,
            last_login,
            ..
        } = self;
        Ok(Accounts {
            user_id,
            username,
            password,
            email,
            created_on,
            last_login,
            _state_marker: PhantomData,
        })
    }
}

#[test]
fn test_orm() {
    let mut client = &mut Config::new()
        .user("postgres")
        .password("postgres")
        .host("127.0.0.1")
        .port(5432)
        .dbname("postgres")
        .connect(NoTls)
        .unwrap();

    let accounts = Accounts::all(&mut client).unwrap();
    assert!(accounts.is_empty());

    let new_account = Accounts::new()
        .with_username("hello".into())
        .with_password("world".into())
        .with_email("user@example.com".into())
        .save(&mut client)
        .unwrap();
    let accounts = Accounts::all(&mut client).unwrap();
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].user_id, new_account.user_id);

    let mut new_account = new_account.edit();
    new_account.set_username("hello2".into());
    let new_account = new_account.save(&mut client).unwrap();
    let accounts = Accounts::all(&mut client).unwrap();
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].user_id, new_account.user_id);
    assert_eq!(&accounts[0].username, "hello2");

    new_account.delete(&mut client).unwrap();
    let accounts = Accounts::all(&mut client).unwrap();
    assert!(accounts.is_empty());
}
