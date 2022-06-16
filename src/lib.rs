//use postgres_types::Json;
//use time::{OffsetDateTime, PrimitiveDateTime};
use heck::{AsSnakeCase, AsUpperCamelCase};
use indexmap::IndexMap;
use std::borrow::Cow;
pub use std::str::FromStr;

mod struct_builder;
pub use struct_builder::*;

mod column;
pub use column::*;

mod types;
pub use types::*;

#[cfg(feature = "postgres")]
pub fn new_struct_builder(client: &mut postgres::Client, table_name: &str) -> Result<StructBuilder, anyhow::Error> {
    let mut struct_bldr = StructBuilder::new(table_name.to_string().into());
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
