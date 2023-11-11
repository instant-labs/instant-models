use std::str::FromStr;
use std::sync::Arc;

use instant_models::{Column, Table, Type};
use similar_asserts::assert_eq;

#[test]
fn basic() -> anyhow::Result<()> {
    let mut table = Table::new("account".into());

    let name = Arc::<str>::from("id");
    let column = Column::new(name.clone(), Type::from_str("integer")?);
    table.columns.insert(name.clone(), column);

    let name = Arc::<str>::from("name");
    let column = Column::new(name.clone(), Type::from_str("text")?);
    table.columns.insert(name.clone(), column);

    let name = Arc::<str>::from("password");
    let column = Column::new(name.clone(), Type::from_str("text")?);
    table.columns.insert(name.clone(), column);

    let name = Arc::<str>::from("email");
    let column = Column::new(name.clone(), Type::from_str("text")?);
    table.columns.insert(name.clone(), column);

    let name = Arc::<str>::from("created_at");
    let column = Column::new(name.clone(), Type::from_str("timestamp with time zone")?);
    table.columns.insert(name.clone(), column);

    let name = Arc::<str>::from("last_login");
    let mut column = Column::new(name.clone(), Type::from_str("timestamp with time zone")?);
    column.null = true;
    table.columns.insert(name.clone(), column);

    assert_eq!(table.to_string(), r#"pub struct Account {
    pub id: i32,
    pub name: String,
    pub password: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct NewAccount<'a> {
    pub id: i32,
    pub name: &'a str,
    pub password: &'a str,
    pub email: &'a str,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
}
"#);

    Ok(())
}
