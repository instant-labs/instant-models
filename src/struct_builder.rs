use crate::{Column, Constraint, NewValue, Type};
use heck::{AsSnakeCase, AsUpperCamelCase};
use indexmap::IndexMap;
use std::borrow::Cow;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct StructBuilder {
    pub name: Cow<'static, str>,
    pub columns: IndexMap<Cow<'static, str>, Column>,
    pub constraints: Vec<Constraint>,
}

impl Default for StructBuilder {
    fn default() -> Self {
        Self {
            name: String::new().into(),
            columns: IndexMap::new(),
            constraints: vec![],
        }
    }
}

impl StructBuilder {
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

    #[cfg(feature = "postgres")]
    pub fn new_from_conn(
        client: &mut postgres::Client,
        table_name: &str,
    ) -> Result<Self, anyhow::Error> {
        let mut struct_bldr = Self::new(table_name.to_string().into());
        let mut col_index: IndexMap<String, Column> = IndexMap::new();
        for row in client.query("SELECT column_name, is_nullable, data_type FROM information_schema.columns WHERE table_name = $1;", &[&table_name])? {
        let column_name: &str = row.get(0);
        let is_nullable: &str = row.get(1);
        let data_type: &str = row.get(2);
        let col = Column::new(column_name.to_string().into(), Type::from_str(data_type)?).set_null(is_nullable == "YES");
        col_index.insert(column_name.to_string(), col);
    }

        for row in client.query("SELECT a.column_name, a.constraint_name, b.constraint_type FROM information_schema.constraint_column_usage AS a JOIN information_schema.table_constraints AS b ON a.constraint_name = b.constraint_name WHERE a.table_name = $1", &[&table_name])? {
        let column_name: &str = row.get(0);
        let constraint_name: &str = row.get(1);
        let constraint_type: &str = row.get(2);
        if let Some(col) = col_index.get_mut(&column_name.to_string()) {
            match constraint_type {
                "UNIQUE" => {col.unique = true;},
                "PRIMARY KEY" => {col.primary_key = true;},
                other => panic!("unknown constraint type: {}", other),
            }
        } else {
            panic!("got constraint for unknown column: column_name {column_name}, constraint_name {constraint_name} constraint_type {constraint_type}");
        }
    }

        for (_, col) in col_index.into_iter() {
            struct_bldr.add_column(col);
        }

        Ok(struct_bldr)
    }
}

impl std::fmt::Display for StructBuilder {
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
