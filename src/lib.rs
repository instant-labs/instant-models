use std::fmt;

use tokio_postgres::Client;

mod struct_builder;
pub use struct_builder::StructBuilder;

mod column;
pub use column::*;

mod types;
pub use types::*;

pub struct Schema {
    pub tables: Vec<StructBuilder>,
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
            let table = StructBuilder::from_postgres(name, &client).await?;
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
