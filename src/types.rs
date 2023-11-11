use std::fmt;
use std::str::FromStr;

use heck::AsUpperCamelCase;
use tokio_postgres::types::Type as PgType;
use tokio_postgres::Client;

use crate::Table;

#[derive(Debug, PartialEq)]
pub enum Type {
    Builtin(PgType),
    Composite(Table),
    Enum(PgEnum),
}

impl Type {
    pub async fn from_postgres(name: &str, client: &Client) -> Result<Self, tokio_postgres::Error> {
        let sql = "SELECT oid, typtype FROM pg_catalog.pg_type WHERE typname = $1";
        let row = client.query_one(sql, &[&name]).await?;
        Ok(match row.get::<_, i8>(1) {
            // enum: 'e' is 101 in ASCII, but b'e' can only be u8 in Rust
            101 => Self::Enum(PgEnum::from_postgres(row.get(0), name, client).await?),
            ty => todo!("unknown Postgres type type {ty:?}"),
        })
    }

    pub fn is_copy(&self) -> bool {
        use Type::*;
        match self {
            Builtin(
                PgType::BOOL
                | PgType::TIMESTAMP
                | PgType::TIMESTAMPTZ
                | PgType::INT8
                | PgType::INT4,
            ) => true,
            Builtin(PgType::TEXT | PgType::TEXT_ARRAY | PgType::BYTEA | PgType::BYTEA_ARRAY) => {
                false
            }
            Composite { .. } => false,
            ty => todo!("{ty:?}::is_copy()"),
        }
    }
}

impl FromStr for Type {
    type Err = anyhow::Error;
    fn from_str(val: &str) -> Result<Self, Self::Err> {
        Ok(Self::Builtin(match val {
            "bigint" => PgType::INT8,
            "integer" => PgType::INT4,
            "text" | "character varying" => PgType::TEXT,
            "text[]" => PgType::TEXT_ARRAY,
            "bytea" => PgType::BYTEA,
            "bytea[]" => PgType::BYTEA_ARRAY,
            "boolean" => PgType::BOOL,
            "timestamp with time zone" => PgType::TIMESTAMPTZ,
            "timestamp without time zone" => PgType::TIMESTAMP,
            _ => todo!("FromStr for {val:?}"),
        }))
    }
}

impl fmt::Display for Type {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use Type::*;
        match self {
            Builtin(PgType::INT8) => write!(fmt, "i64"),
            Builtin(PgType::INT4) => write!(fmt, "i32"),
            Builtin(PgType::TEXT) => write!(fmt, "String"),
            Builtin(PgType::TEXT_ARRAY) => write!(fmt, "Vec<String>"),
            Builtin(PgType::BYTEA) => write!(fmt, "Vec<u8>"),
            Builtin(PgType::BYTEA_ARRAY) => write!(fmt, "Vec<Vec<u8>>"),
            Builtin(PgType::BOOL) => write!(fmt, "bool"),
            Builtin(PgType::TIMESTAMP) => write!(fmt, "chrono::naive::NaiveDateTime"),
            Builtin(PgType::TIMESTAMPTZ) => write!(fmt, "chrono::DateTime<chrono::Utc>"),
            Composite(inner) => write!(fmt, "{}", AsUpperCamelCase(&inner.name)),
            ty => todo!("fmt::Display for {ty:?}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PgEnum {
    name: String,
    variants: Vec<String>,
}

impl PgEnum {
    async fn from_postgres(
        id: u32,
        name: &str,
        client: &Client,
    ) -> Result<Self, tokio_postgres::Error> {
        let mut new = Self {
            name: name.to_owned(),
            variants: Vec::new(),
        };

        let sql = r#"
            SELECT enumlabel
            FROM pg_catalog.pg_enum
            WHERE enumtypid = $1
            ORDER BY enumsortorder ASC
        "#;
        for row in client.query(sql, &[&id]).await? {
            let label = row.get::<_, &str>(0);
            new.variants.push(label.to_owned());
        }

        Ok(new)
    }
}

impl fmt::Display for PgEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("enum {} {{\n", AsUpperCamelCase(&self.name)))?;
        for variant in &self.variants {
            f.write_fmt(format_args!("    {},", AsUpperCamelCase(&variant)))?;
        }

        f.write_str("}\n")
    }
}

pub struct TypeAsRef<'a> {
    pub lifetime: Option<&'a str>,
    pub val: &'a Type,
}

impl fmt::Display for TypeAsRef<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let Self { val, lifetime } = self;
        let (lt_prefix, lt_name, lt_suffix) = match lifetime {
            Some(lt) => ("'", *lt, " "),
            None => ("", "", ""),
        };

        use Type::*;
        match val {
            Builtin(PgType::INT8) => write!(fmt, "i64"),
            Builtin(PgType::INT4) => write!(fmt, "i32"),
            Builtin(PgType::TEXT) => write!(fmt, "&{}{}{}str", lt_prefix, lt_name, lt_suffix),
            Builtin(PgType::TEXT_ARRAY) => {
                write!(fmt, "Vec<&{}{}{}str>", lt_prefix, lt_name, lt_suffix)
            }
            Builtin(PgType::BYTEA) => write!(fmt, "Vec<u8>"),
            Builtin(PgType::BYTEA_ARRAY) => {
                write!(fmt, "Vec<&{}{}{}[u8]>", lt_prefix, lt_name, lt_suffix)
            }
            Builtin(PgType::BOOL) => write!(fmt, "bool"),
            Builtin(PgType::TIMESTAMP) => write!(fmt, "chrono::naive::NaiveDateTime",),
            Builtin(PgType::TIMESTAMPTZ) => write!(fmt, "chrono::DateTime<chrono::Utc>",),
            Builtin(inner) => todo!("no Display for {inner:?}"),
            Composite(inner) => write!(
                fmt,
                "&{}{}{}{}",
                lt_prefix,
                lt_name,
                lt_suffix,
                AsUpperCamelCase(&inner.name)
            ),
            Enum(inner) => fmt.write_fmt(format_args!("{inner}")),
        }
    }
}
