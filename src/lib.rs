use std::fmt;

use tokio_postgres::Client;

mod table;
pub use table::Table;
mod column;
pub use column::Column;
mod types;
pub use types::Type;

pub struct Schema {
    pub tables: Vec<Table>,
}

impl Schema {
    #[cfg(feature = "postgres")]
    pub async fn from_postgres(client: &Client) -> Result<Self, anyhow::Error> {
        let sql = r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = 'public'
        "#;

        let mut tables = Vec::new();
        for row in client.query(sql, &[]).await? {
            let name = row.get::<_, &str>(0);
            let table = Table::from_postgres(name, client).await?;
            tables.push(table);
        }

        Ok(Self { tables })
    }
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for table in &self.tables {
            f.write_fmt(format_args!("{table}"))?;
        }

        Ok(())
    }
}
