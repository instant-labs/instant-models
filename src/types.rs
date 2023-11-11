use crate::StructBuilder;
use heck::AsUpperCamelCase;
use postgres_types::Type as PgType;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Type {
    Builtin { inner: PgType },
    Composite { inner: StructBuilder },
}

impl FromStr for Type {
    type Err = anyhow::Error;
    fn from_str(val: &str) -> Result<Self, Self::Err> {
        Ok(Self::Builtin {
            inner: match val {
                "bigint" => PgType::INT8,
                "integer" => PgType::INT4,
                "text" => PgType::TEXT,
                "text[]" => PgType::TEXT_ARRAY,
                "bytea" => PgType::BYTEA,
                "bytea[]" => PgType::BYTEA_ARRAY,
                "boolean" => PgType::BOOL,
                "character varying" => PgType::TEXT,
                "timestamp with time zone" => PgType::TIMESTAMPTZ,
                "timestamp without time zone" => PgType::TIMESTAMP,
                _ => todo!("FromStr for {val:?}"),
            },
        })
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Builtin {
                inner: PgType::INT8,
            } => write!(fmt, "i64"),
            Self::Builtin {
                inner: PgType::INT4,
            } => write!(fmt, "i32"),
            Self::Builtin {
                inner: PgType::TEXT,
            } => write!(fmt, "String"),
            Self::Builtin {
                inner: PgType::TEXT_ARRAY,
            } => write!(fmt, "Vec<String>"),
            Self::Builtin {
                inner: PgType::BYTEA,
            } => write!(fmt, "Vec<u8>"),
            Self::Builtin {
                inner: PgType::BYTEA_ARRAY,
            } => write!(fmt, "Vec<Vec<u8>>"),
            Self::Builtin {
                inner: PgType::BOOL,
            } => write!(fmt, "bool"),
            Self::Builtin {
                inner: PgType::TIMESTAMP,
            } => write!(fmt, "chrono::naive::NaiveDateTime"),
            Self::Builtin {
                inner: PgType::TIMESTAMPTZ,
            } => write!(fmt, "chrono::DateTime<chrono::Utc>"),
            Self::Composite { inner } => write!(fmt, "{}", AsUpperCamelCase(&inner.name)),
            ty => todo!("fmt::Display for {ty:?}"),
        }
    }
}

impl Type {
    pub fn is_copy(&self) -> bool {
        match self {
            Self::Builtin {
                inner: PgType::BOOL,
            }
            | Self::Builtin {
                inner: PgType::TIMESTAMP,
            }
            | Self::Builtin {
                inner: PgType::TIMESTAMPTZ,
            }
            | Self::Builtin {
                inner: PgType::INT8,
            }
            | Self::Builtin {
                inner: PgType::INT4,
            } => true,
            Self::Builtin {
                inner: PgType::TEXT,
            }
            | Self::Builtin {
                inner: PgType::TEXT_ARRAY,
            }
            | Self::Builtin {
                inner: PgType::BYTEA,
            }
            | Self::Builtin {
                inner: PgType::BYTEA_ARRAY,
            }
            | Self::Composite { inner: _ } => false,
            ty => todo!("{ty:?}::is_copy()"),
        }
    }
}

pub struct TypeAsRef<'a> {
    pub lifetime: Option<&'a str>,
    pub val: &'a Type,
}

impl std::fmt::Display for TypeAsRef<'_> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let Self { val, lifetime } = self;
        match val {
            Type::Builtin {
                inner: PgType::INT8,
            } => write!(fmt, "i64"),
            Type::Builtin {
                inner: PgType::INT4,
            } => write!(fmt, "i32"),
            Type::Builtin {
                inner: PgType::TEXT,
            } => write!(
                fmt,
                "&{}{}{}str",
                if lifetime.is_some() { "'" } else { "" },
                if let Some(l) = lifetime.as_ref() {
                    *l
                } else {
                    ""
                },
                if lifetime.is_some() { " " } else { "" }
            ),
            Type::Builtin {
                inner: PgType::TEXT_ARRAY,
            } => write!(
                fmt,
                "Vec<&{}{}{}str>",
                if lifetime.is_some() { "'" } else { "" },
                if let Some(l) = lifetime.as_ref() {
                    *l
                } else {
                    ""
                },
                if lifetime.is_some() { " " } else { "" }
            ),
            Type::Builtin {
                inner: PgType::BYTEA,
            } => write!(fmt, "Vec<u8>"),
            Type::Builtin {
                inner: PgType::BYTEA_ARRAY,
            } => write!(
                fmt,
                "Vec<&{}{}{}[u8]>",
                if lifetime.is_some() { "'" } else { "" },
                if let Some(l) = lifetime.as_ref() {
                    *l
                } else {
                    ""
                },
                if lifetime.is_some() { " " } else { "" }
            ),
            Type::Builtin {
                inner: PgType::BOOL,
            } => write!(fmt, "bool"),
            Type::Builtin {
                inner: PgType::TIMESTAMP,
            } => write!(fmt, "chrono::naive::NaiveDateTime",),
            Type::Builtin {
                inner: PgType::TIMESTAMPTZ,
            } => write!(fmt, "chrono::DateTime<chrono::Utc>",),
            Type::Composite { inner } => write!(
                fmt,
                "&{}{}{}{}",
                if lifetime.is_some() { "'" } else { "" },
                if let Some(l) = lifetime.as_ref() {
                    *l
                } else {
                    ""
                },
                if lifetime.is_some() { " " } else { "" },
                AsUpperCamelCase(&inner.name)
            ),
            _ => todo!(),
        }
    }
}
