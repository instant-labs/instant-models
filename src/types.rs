use super::*;

#[derive(Debug, PartialEq)]
pub enum Type {
    Builtin { inner: postgres_types::Type },
    Composite { inner: StructBuilder },
}

impl FromStr for Type {
    type Err = anyhow::Error;
    fn from_str(val: &str) -> Result<Self, Self::Err> {
        Ok(Self::Builtin {
            inner: match val {
                "bigint" => postgres_types::Type::INT8,
                "integer" => postgres_types::Type::INT4,
                "text" => postgres_types::Type::TEXT,
                "text[]" => postgres_types::Type::TEXT_ARRAY,
                "bytea" => postgres_types::Type::BYTEA,
                "bytea[]" => postgres_types::Type::BYTEA_ARRAY,
                "boolean" => postgres_types::Type::BOOL,
                "character varying" => postgres_types::Type::TEXT,
                "timestamp with time zone" => postgres_types::Type::TIMESTAMPTZ,
                "timestamp without time zone" => postgres_types::Type::TIMESTAMP,
                _ => todo!(),
            },
        })
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Builtin {
                inner: postgres_types::Type::INT8,
            } => write!(fmt, "i64"),
            Self::Builtin {
                inner: postgres_types::Type::INT4,
            } => write!(fmt, "i32"),
            Self::Builtin {
                inner: postgres_types::Type::TEXT,
            } => write!(fmt, "String"),
            Self::Builtin {
                inner: postgres_types::Type::TEXT_ARRAY,
            } => write!(fmt, "Vec<String>"),
            Self::Builtin {
                inner: postgres_types::Type::BYTEA,
            } => write!(fmt, "Vec<u8>"),
            Self::Builtin {
                inner: postgres_types::Type::BYTEA_ARRAY,
            } => write!(fmt, "Vec<Vec<u8>>"),
            Self::Builtin {
                inner: postgres_types::Type::BOOL,
            } => write!(fmt, "bool"),
            Self::Builtin {
                inner: postgres_types::Type::TIMESTAMP,
            } => write!(fmt, "chrono::naive::NaiveDateTime"),
            Self::Builtin {
                inner: postgres_types::Type::TIMESTAMPTZ,
            } => write!(fmt, "chrono::DateTime<chrono::Utc>"),
            Self::Composite { inner } => write!(fmt, "{}", AsUpperCamelCase(&inner.name)),
            _ => todo!(),
        }
    }
}

impl Type {
    pub fn is_copy(&self) -> bool {
        match self {
            Self::Builtin {
                inner: postgres_types::Type::BOOL,
            }
            | Self::Builtin {
                inner: postgres_types::Type::INT8,
            }
            | Self::Builtin {
                inner: postgres_types::Type::INT4,
            } => true,
            Self::Builtin {
                inner: postgres_types::Type::TEXT,
            }
            | Self::Builtin {
                inner: postgres_types::Type::TEXT_ARRAY,
            }
            | Self::Builtin {
                inner: postgres_types::Type::BYTEA,
            }
            | Self::Builtin {
                inner: postgres_types::Type::BYTEA_ARRAY,
            }
            | Self::Builtin {
                inner: postgres_types::Type::TIMESTAMP,
            }
            | Self::Builtin {
                inner: postgres_types::Type::TIMESTAMPTZ,
            }
            | Self::Composite { inner: _ } => false,
            _ => todo!(),
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
                inner: postgres_types::Type::INT8,
            } => write!(fmt, "i64"),
            Type::Builtin {
                inner: postgres_types::Type::INT4,
            } => write!(fmt, "i32"),
            Type::Builtin {
                inner: postgres_types::Type::TEXT,
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
                inner: postgres_types::Type::TEXT_ARRAY,
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
                inner: postgres_types::Type::BYTEA,
            } => write!(fmt, "Vec<u8>"),
            Type::Builtin {
                inner: postgres_types::Type::BYTEA_ARRAY,
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
                inner: postgres_types::Type::BOOL,
            } => write!(fmt, "bool"),
            Type::Builtin {
                inner: postgres_types::Type::TIMESTAMP,
            } => write!(
                fmt,
                "&{}{}{}chrono::naive::NaiveDateTime",
                if lifetime.is_some() { "'" } else { "" },
                if let Some(l) = lifetime.as_ref() {
                    *l
                } else {
                    ""
                },
                if lifetime.is_some() { " " } else { "" }
            ),
            Type::Builtin {
                inner: postgres_types::Type::TIMESTAMPTZ,
            } => write!(
                fmt,
                "&{}{}{}chrono::DateTime<chrono::Utc>",
                if lifetime.is_some() { "'" } else { "" },
                if let Some(l) = lifetime.as_ref() {
                    *l
                } else {
                    ""
                },
                if lifetime.is_some() { " " } else { "" }
            ),
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
