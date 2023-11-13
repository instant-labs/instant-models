use std::fmt;
#[cfg(feature = "postgres")]
use std::str::FromStr;
use std::sync::Arc;

use heck::{AsSnakeCase, AsUpperCamelCase};
use indexmap::IndexMap;
#[cfg(feature = "postgres")]
use tokio_postgres::Client;

#[cfg(feature = "postgres")]
use crate::column::{Column, Constraint, ForeignKey};
use crate::types::Type;

#[derive(Debug, PartialEq)]
pub struct Table {
    pub name: Arc<str>,
    pub columns: IndexMap<Arc<str>, Column>,
    pub constraints: Vec<Constraint>,
}

impl Table {
    #[cfg(feature = "postgres")]
    pub async fn from_postgres(name: &str, client: &Client) -> anyhow::Result<Self> {
        let sql = r#"
            SELECT column_name, data_type, is_nullable, udt_name
            FROM information_schema.columns
            WHERE table_name = $1
        "#;
        let mut new = Table::new(name.to_owned().into());
        for row in client.query(sql, &[&name]).await? {
            let name = Arc::<str>::from(row.get::<_, &str>(0));
            let r#type = match row.get::<_, &str>(1) {
                "ARRAY" | "USER-DEFINED" => Type::from_postgres_by_name(row.get(3), client).await?,
                other => Type::from_str(other)?,
            };

            let mut column = Column::new(name.clone(), r#type);
            column.null = row.get::<_, &str>(2) == "YES";
            new.columns.insert(name, column);
        }

        let sql = r#"
            SELECT
                usage.constraint_name, usage.column_name,
                constraints.constraint_name, constraints.constraint_type
            FROM information_schema.constraint_column_usage AS usage
                JOIN information_schema.table_constraints AS constraints
                    ON usage.constraint_name = constraints.constraint_name
            WHERE usage.table_name = $1
        "#;
        for row in client.query(sql, &[&name]).await? {
            let name = row.get::<_, &str>(0);
            let column = row.get::<_, &str>(1);
            let column = match new.columns.get_mut(column) {
                Some(col) => col,
                None => panic!("constraint {name:?} for unknown column {column:?}"),
            };

            match row.get::<_, &str>(3) {
                "UNIQUE" => column.unique = true,
                "PRIMARY KEY" => column.primary_key = true,
                "FOREIGN KEY" => {
                    column.foreign_key = Some(ForeignKey::from_postgres(row.get(2), client).await?);
                }
                other => panic!("unknown constraint type {other:?}"),
            }
        }

        Ok(new)
    }

    pub fn new(name: Arc<str>) -> Self {
        Self {
            name,
            columns: IndexMap::default(),
            constraints: Vec::default(),
        }
    }
}

impl fmt::Display for Table {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let ty_name = AsUpperCamelCase(&self.name);
        fmt.write_fmt(format_args!("pub struct {ty_name} {{\n"))?;
        for col in self.columns.values() {
            fmt.write_fmt(format_args!("    pub {col},\n"))?;
        }
        fmt.write_str("}\n\n")?;

        // Multi-value `insert()` method
        fmt.write_fmt(format_args!(
            r#"impl {ty_name} {{
    pub async fn insert(slice: &[New{ty_name}<'_>]) -> Result<(), tokio_postgres::Error> {{
        let statement = client.prepare(
            "INSERT INTO {} (
"#,
            &self.name
        ))?;

        let num = self.columns.len();
        for (i, col) in self.columns.values().enumerate() {
            if col.name.as_ref() == "id" {
                continue;
            }

            fmt.write_fmt(format_args!("                {}", col.name))?;
            if i == num - 1 {
                fmt.write_str("\n")?;
            } else {
                fmt.write_str(",\n")?;
            }
        }

        fmt.write_str("            ) VALUES (")?;
        let mut skipped = 0;
        for (i, col) in self.columns.values().enumerate() {
            if col.name.as_ref() == "id" {
                skipped += 1;
                continue;
            }

            let idx = i + 1 - skipped;
            if i == num - 1 {
                fmt.write_fmt(format_args!("${idx}"))?;
            } else {
                fmt.write_fmt(format_args!("${idx}, "))?;
            }
        }
        fmt.write_str(
            r#")"
        ).await?;
        for entry in slice {
            client.execute(&statement, &[
"#,
        )?;

        for (i, col) in self.columns.values().enumerate() {
            if col.name.as_ref() == "id" {
                continue;
            }

            if i == num - 1 {
                fmt.write_fmt(format_args!(
                    "                entry.{},\n",
                    AsSnakeCase(&col.name)
                ))?;
            } else {
                fmt.write_fmt(format_args!(
                    "                entry.{},\n",
                    AsSnakeCase(&col.name)
                ))?;
            }
        }

        fmt.write_str(
            "            ]).await?;
        }

        Ok(())
    }
}\n",
        )?;

        // `New` initialization type
        fmt.write_fmt(format_args!("\npub struct New{ty_name}<'a> {{\n",))?;
        for col in self.columns.values() {
            fmt.write_fmt(format_args!("    pub {},\n", col.new_field()))?;
        }

        fmt.write_str("}\n")
    }
}
