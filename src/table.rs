use std::borrow::Cow;
#[cfg(feature = "postgres")]
use std::str::FromStr;

use heck::{AsSnakeCase, AsUpperCamelCase};
use indexmap::IndexMap;
#[cfg(feature = "postgres")]
use tokio_postgres::Client;

#[cfg(feature = "postgres")]
use crate::column::{Column, Constraint, NewValue};
use crate::types::Type;

#[derive(Debug, Default, PartialEq)]
pub struct Table {
    pub name: Cow<'static, str>,
    pub columns: IndexMap<Cow<'static, str>, Column>,
    pub constraints: Vec<Constraint>,
}

impl Table {
    #[cfg(feature = "postgres")]
    pub async fn from_postgres(name: &str, client: &Client) -> anyhow::Result<Self> {
        let sql = r#"
            SELECT column_name, is_nullable, data_type
            FROM information_schema.columns
            WHERE table_name = $1
        "#;
        let mut new = Table::new(name.to_owned().into());
        for row in client.query(sql, &[&name]).await? {
            let name = row.get::<_, &str>(0);
            let nullable = row.get::<_, &str>(1);
            let data_type = row.get::<_, &str>(2);
            let col = Column::new(name.to_owned().into(), Type::from_str(data_type)?)
                .set_null(nullable == "YES");
            new.columns.insert(name.to_owned().into(), col);
        }

        let sql = r#"
            SELECT usage.constraint_name, usage.column_name, constraints.constraint_type
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

            match row.get::<_, &str>(2) {
                "UNIQUE" => column.unique = true,
                "PRIMARY KEY" => column.primary_key = true,
                other => panic!("unknown constraint type {other:?}"),
            }
        }

        Ok(new)
    }

    pub fn new(name: Cow<'static, str>) -> Self {
        Self {
            name,
            ..Self::default()
        }
    }

    pub fn add_column(&mut self, val: Column) -> &mut Self {
        self.columns.insert(val.name.clone(), val);
        self
    }

    pub fn build_type(&self) -> String {
        format!("{}", self)
    }

    pub fn build_new_type(&self) -> String {
        let columns =
            self.columns
                .values()
                .filter(|c| !c.primary_key)
                .fold(String::new(), |mut acc, col| {
                    acc.push_str(&format!(
                        "    pub {},",
                        NewValue {
                            val: col,
                            lifetime: Some("a")
                        }
                    ));
                    acc.push('\n');
                    acc
                });

        format!(
            r#"pub struct {}New<'a> {{
{}}}
        "#,
            AsUpperCamelCase(&self.name),
            columns
        )
    }

    pub fn build_type_methods(&self) -> String {
        let mut sql_statement = format!("INSERT INTO {}(", self.name);
        let parameters = self
            .columns
            .values()
            .filter(|c| !c.primary_key)
            .map(|c| c.name.as_ref())
            .collect::<Vec<&str>>();
        for p in parameters.iter() {
            sql_statement.push_str(p);
            sql_statement.push_str(", ");
        }
        if sql_statement.ends_with(", ") {
            sql_statement.pop();
            sql_statement.pop();
        }
        sql_statement.push_str(") VALUES(");
        for i in 0..parameters.len() {
            sql_statement.push_str(&format!("${}, ", i + 1));
        }
        if sql_statement.ends_with(", ") {
            sql_statement.pop();
            sql_statement.pop();
        }
        sql_statement.push_str(");");
        let mut fields =
            self.columns
                .values()
                .filter(|c| !c.primary_key)
                .fold(String::new(), |mut acc, col| {
                    acc.push_str(&format!("&entry.{}, ", AsSnakeCase(&col.name)));
                    acc
                });
        if fields.ends_with(", ") {
            fields.pop();
            fields.pop();
        }
        format!(
            r#"impl {0} {{
        pub fn insert_slice(client: &mut postgres::Client, slice: &[{0}New<'_>]) -> Result<(), postgres::Error> {{
            let statement = client.prepare("{sql_statement}")?;
            for entry in slice {{
                client.execute(&statement, &[{fields}])?;
            }}
            Ok(())
        }}
}}
        "#,
            AsUpperCamelCase(&self.name),
        )
    }

    /*
        pub fn build_new_type_methods(&self) -> String {
            let lifetime: &str = "a";
            let mut parameters =
                self.columns
                    .values()
                    .filter(|c| !c.primary_key)
                    .fold(String::new(), |mut acc, col| {
                        acc.push_str(&format!(
                            "{}",
                            NewValue {
                                val: col,
                                lifetime: Some(lifetime)
                            }
                        ));
                        acc.push_str(", ");
                        acc
                    });
            if parameters.ends_with(", ") {
                parameters.pop();
                parameters.pop();
            }

            let mut fields =
                self.columns
                    .values()
                    .filter(|c| !c.primary_key)
                    .fold(String::new(), |mut acc, col| {
                        acc.push_str(&format!("{}, ", AsSnakeCase(&col.name)));
                        acc
                    });
            if fields.ends_with(", ") {
                fields.pop();
                fields.pop();
            }

            format!(
                r#"impl<'{lifetime}> {}New<'{lifetime}> {{
        pub fn new({parameters}) -> Self {{
            Self {{ {fields} }}
        }}
    }}
            "#,
                AsUpperCamelCase(&self.name),
            )
        }
    */
}

impl std::fmt::Display for Table {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let columns = self.columns.values().fold(String::new(), |mut acc, col| {
            acc.push_str(&format!("    pub {},\n", col));
            acc
        });
        write!(
            fmt,
            r#"pub struct {} {{
{}}}
        "#,
            AsUpperCamelCase(&self.name),
            columns
        )
    }
}
