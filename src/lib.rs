use std::fmt;

use tokio_postgres::Client;

mod table;
pub use table::Table;
mod column;
pub use column::Column;
mod types;
pub use types::{Enum, Type, TypeDefinition, Composite};

pub struct Schema {
    pub tables: Vec<Table>,
    pub types: Vec<TypeDefinition>,
}

impl Schema {
    #[cfg(feature = "postgres")]
    pub async fn from_postgres(client: &Client) -> anyhow::Result<Self> {
        let sql = r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = 'public'
        "#;

        let (mut tables, mut types) = (Vec::new(), Vec::new());
        for row in client.query(sql, &[]).await? {
            let name = row.get::<_, &str>(0);
            let table = Table::from_postgres(name, client).await?;

            for column in table.columns.values() {
                match &column.r#type {
                    Type::Composite(c) => types.push(TypeDefinition::Composite(
                        Composite::from_postgres(&c.name, client).await?,
                    )),
                    Type::Enum(e) => types.push(TypeDefinition::Enum(
                        Enum::from_postgres(&e.name, client).await?,
                    )),
                    _ => {}
                }
            }

            tables.push(table);
        }

        Ok(Self { tables, types })
    }
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for table in &self.tables {
            f.write_fmt(format_args!("{table}"))?;
            f.write_str("\n")?;
        }

        for type_def in &self.types {
            f.write_fmt(format_args!("{type_def}"))?;
            f.write_str("\n")?;
        }

        Ok(())
    }
}
