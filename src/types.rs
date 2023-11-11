use std::fmt;
use std::str::FromStr;

use heck::AsUpperCamelCase;
use tokio_postgres::types::Type as PgType;

use crate::Table;

#[derive(Debug, PartialEq)]
pub enum Type {
    Builtin(PgType),
    Composite(Table),
}

impl Type {
    pub fn is_copy(&self) -> bool {
        match self {
            Self::Builtin(
                PgType::BOOL
                | PgType::TIMESTAMP
                | PgType::TIMESTAMPTZ
                | PgType::INT8
                | PgType::INT4,
            ) => true,
            Self::Builtin(
                PgType::TEXT | PgType::TEXT_ARRAY | PgType::BYTEA | PgType::BYTEA_ARRAY,
            ) => false,
            Self::Composite { .. } => false,
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
        match self {
            Self::Builtin(PgType::INT8) => write!(fmt, "i64"),
            Self::Builtin(PgType::INT4) => write!(fmt, "i32"),
            Self::Builtin(PgType::TEXT) => write!(fmt, "String"),
            Self::Builtin(PgType::TEXT_ARRAY) => write!(fmt, "Vec<String>"),
            Self::Builtin(PgType::BYTEA) => write!(fmt, "Vec<u8>"),
            Self::Builtin(PgType::BYTEA_ARRAY) => write!(fmt, "Vec<Vec<u8>>"),
            Self::Builtin(PgType::BOOL) => write!(fmt, "bool"),
            Self::Builtin(PgType::TIMESTAMP) => write!(fmt, "chrono::naive::NaiveDateTime"),
            Self::Builtin(PgType::TIMESTAMPTZ) => write!(fmt, "chrono::DateTime<chrono::Utc>"),
            Self::Composite(inner) => write!(fmt, "{}", AsUpperCamelCase(&inner.name)),
            ty => todo!("fmt::Display for {ty:?}"),
        }
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

        match val {
            Type::Builtin(PgType::INT8) => write!(fmt, "i64"),
            Type::Builtin(PgType::INT4) => write!(fmt, "i32"),
            Type::Builtin(PgType::TEXT) => write!(fmt, "&{}{}{}str", lt_prefix, lt_name, lt_suffix),
            Type::Builtin(PgType::TEXT_ARRAY) => {
                write!(fmt, "Vec<&{}{}{}str>", lt_prefix, lt_name, lt_suffix)
            }
            Type::Builtin(PgType::BYTEA) => write!(fmt, "Vec<u8>"),
            Type::Builtin(PgType::BYTEA_ARRAY) => {
                write!(fmt, "Vec<&{}{}{}[u8]>", lt_prefix, lt_name, lt_suffix)
            }
            Type::Builtin(PgType::BOOL) => write!(fmt, "bool"),
            Type::Builtin(PgType::TIMESTAMP) => write!(fmt, "chrono::naive::NaiveDateTime",),
            Type::Builtin(PgType::TIMESTAMPTZ) => write!(fmt, "chrono::DateTime<chrono::Utc>",),
            Type::Builtin(inner) => todo!("no Display for {inner:?}"),
            Type::Composite(inner) => write!(
                fmt,
                "&{}{}{}{}",
                lt_prefix,
                lt_name,
                lt_suffix,
                AsUpperCamelCase(&inner.name)
            ),
        }
    }
}
