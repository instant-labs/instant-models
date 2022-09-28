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
{}}}"#,
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
            sql_statement.push_str(*p);
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
}}"#,
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

    /// Generates a helper enum and struct to allow accessing field identifiers when building
    /// SQL queries.
    #[cfg(feature = "sql")]
    pub fn build_field_identifiers(&self) -> String {
        let mut output: String = String::new();

        // Generate enum with sea_query field identifiers: `<NAME>Iden`.
        // TODO: use a proc-macro to derive this instead?
        // TODO: replace sea_query.
        let struct_name = format!("{}", AsUpperCamelCase(&self.name));
        let enum_name = format!("{}Iden", struct_name);
        let column_names = self.columns.values().fold(String::new(), |mut acc, col| {
            acc.push_str(&format!("    {},\n", AsUpperCamelCase(&col.name)));
            acc
        });
        let match_iden_columns = self.columns.values().fold(String::new(), |mut acc, col| {
            acc.push_str(&format!(
                "                Self::{} => \"{}\",\n",
                AsUpperCamelCase(&col.name),
                &col.name
            ));
            acc
        });
        output.push_str(&format!(
            r#"
#[derive(Copy, Clone)]
pub enum {} {{
    Table,
{}}}

impl sea_query::Iden for {} {{
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {{
        write!(
            s,
            "{{}}",
            match self {{
                Self::Table => "{}",
{}            }}).expect("{} failed to write");
    }}
}}
"#,
            enum_name, column_names, enum_name, &self.name, match_iden_columns, enum_name,
        ));

        // Generate fields struct: `<NAME>Fields`.
        // TODO: derive with proc-macro instead?
        let fields_name = format!("{}Fields", AsUpperCamelCase(&self.name));
        let fields = self.columns.values().fold(String::new(), |mut acc, col| {
            if col.null {
                acc.push_str(&format!(
                    "    pub {}: ::instant_models::Field<Option<{}>, {}>,\n",
                    AsSnakeCase(&col.name),
                    col.r#type,
                    enum_name
                ));
            } else {
                acc.push_str(&format!(
                    "    pub {}: ::instant_models::Field<{}, {}>,\n",
                    AsSnakeCase(&col.name),
                    col.r#type,
                    enum_name
                ));
            }
            acc
        });
        output.push_str(&format!(
            r#"
pub struct {} {{
{}}}
"#,
            fields_name, fields,
        ));

        // Implement Table for the struct.
        // TODO: derive with proc-macro instead?
        let fields_instance = self.columns.values().fold(String::new(), |mut acc, col| {
            acc.push_str(&format!(
                "        {}: ::instant_models::Field::new(\"{}\", {}::{}),\n",
                AsSnakeCase(&col.name),
                col.name,
                enum_name,
                AsUpperCamelCase(&col.name)
            ));
            acc
        });
        output.push_str(&format!(
            r#"
impl instant_models::Table for {} {{
    type FieldsType = {};
    const FIELDS: Self::FieldsType = {} {{
{}    }};

    fn table() -> sea_query::TableRef {{
        use sea_query::IntoTableRef;
        {}::Table.into_table_ref()
    }}
}}"#,
            struct_name, fields_name, fields_name, fields_instance, enum_name,
        ));

        output
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
{}}}"#,
            AsUpperCamelCase(&self.name),
            columns
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_struct_builder() -> StructBuilder {
        let mut columns = IndexMap::new();
        columns.insert(
            "user_id".into(),
            Column::new("user_id".into(), Type::from_str("integer").unwrap()),
        );
        columns.insert(
            "username".into(),
            Column::new("username".into(), Type::from_str("text").unwrap()),
        );
        columns.insert(
            "email".into(),
            Column::new("email".into(), Type::from_str("text").unwrap()).set_null(true),
        );

        let constraints = vec![Constraint::PrimaryKey {
            name: "pk_user_id".into(),
            columns: vec!["user_id".into()],
        }];

        StructBuilder {
            name: "accounts".into(),
            columns,
            constraints,
        }
    }

    #[test]
    fn test_build_field_identifiers() {
        let builder: StructBuilder = mock_struct_builder();
        assert_eq!(
            builder.build_field_identifiers(),
            r##"
#[derive(Copy, Clone)]
pub enum AccountsIden {
    Table,
    UserId,
    Username,
    Email,
}

impl sea_query::Iden for AccountsIden {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "accounts",
                Self::UserId => "user_id",
                Self::Username => "username",
                Self::Email => "email",
            }).expect("AccountsIden failed to write");
    }
}

pub struct AccountsFields {
    pub user_id: ::instant_models::Field<i32, AccountsIden>,
    pub username: ::instant_models::Field<String, AccountsIden>,
    pub email: ::instant_models::Field<Option<String>, AccountsIden>,
}

impl instant_models::Table for Accounts {
    type FieldsType = AccountsFields;
    const FIELDS: Self::FieldsType = AccountsFields {
        user_id: ::instant_models::Field::new("user_id", AccountsIden::UserId),
        username: ::instant_models::Field::new("username", AccountsIden::Username),
        email: ::instant_models::Field::new("email", AccountsIden::Email),
    };

    fn table() -> sea_query::TableRef {
        use sea_query::IntoTableRef;
        AccountsIden::Table.into_table_ref()
    }
}"##
        );
    }
}
