use std::fmt;
use std::str::FromStr;

use async_recursion::async_recursion;
use heck::{AsUpperCamelCase, AsSnakeCase};
use tokio_postgres::types::{ToSql, Type as PgType};
use tokio_postgres::Client;

#[derive(Debug, PartialEq)]
pub enum Type {
    Array(Box<Type>),
    Builtin(PgType),
    Composite(CompositeRef),
    Enum(EnumRef),
    Vector,
}

impl Type {
    pub async fn from_postgres_by_name(
        name: &str,
        client: &Client,
    ) -> Result<Self, tokio_postgres::Error> {
        Self::from_postgres("typname = $1", &[&name], client).await
    }

    #[async_recursion]
    pub async fn from_postgres_by_id(
        oid: u32,
        client: &Client,
    ) -> Result<Self, tokio_postgres::Error> {
        match PgType::from_oid(oid) {
            Some(ty) => Ok(Self::Builtin(ty)),
            None => Self::from_postgres("oid = $1", &[&oid], client).await,
        }
    }

    async fn from_postgres(
        cond: &str,
        args: &[&(dyn ToSql + Sync)],
        client: &Client,
    ) -> Result<Self, tokio_postgres::Error> {
        let sql = format!(
            "SELECT oid, typname, typtype, typrelid, typelem
            FROM pg_catalog.pg_type WHERE {cond}"
        );
        let rows = client.query(&sql, args).await?;
        let row = match &rows[..] {
            [row] => row,
            [] => todo!("no Postgres type found for {cond} with {args:?}"),
            _ => todo!("multiple Postgres types found for {cond} with {args:?}"),
        };

        // A Postgres 'char' is represented as an `u8`
        Ok(match row.get::<_, i8>(2) {
            // array: 'b' is 98 in ASCII
            98 => match row.get(4) {
                0 => return Ok(Self::from_str(row.get::<_, &str>(1)).unwrap()),
                oid => Self::Array(Self::from_postgres_by_id(oid, client).await?.into()),
            },
            // composite: 'c' is 99 in ASCII
            99 => {
                return Ok(Self::Composite(CompositeRef {
                    name: row.get::<_, &str>(1).to_owned(),
                }))
            }
            // enum: 'e' is 101 in ASCII
            101 => Self::Enum(EnumRef {
                name: row.get::<_, &str>(1).to_owned(),
            }),
            ty => todo!(
                "unknown Postgres type {ty:?} (from name {:?})",
                row.get::<_, &str>(1)
            ),
        })
    }

    pub fn is_copy(&self) -> bool {
        use Type::*;
        match self {
            Builtin(
                PgType::BOOL
                | PgType::FLOAT4
                | PgType::FLOAT8
                | PgType::TIMESTAMP
                | PgType::TIMESTAMPTZ
                | PgType::INT8
                | PgType::INT4
                | PgType::INT2
                | PgType::INET
                | PgType::INTERVAL,
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
        if val == "vector" || val == "tsvector" {
            return Ok(Self::Vector);
        }

        Ok(Self::Builtin(match val {
            "bigint" => PgType::INT8,
            "boolean" => PgType::BOOL,
            "bytea" => PgType::BYTEA,
            "bytea[]" => PgType::BYTEA_ARRAY,
            "float4" | "real" => PgType::FLOAT4,
            "float8" | "double precision" => PgType::FLOAT8,
            "inet" => PgType::INET,
            "smallint" | "int2" => PgType::INT2,
            "integer" | "int4" => PgType::INT4,
            "interval" => PgType::INTERVAL,
            "jsonb" => PgType::JSONB,
            "text" | "character varying" => PgType::TEXT,
            "text[]" => PgType::TEXT_ARRAY,
            "timestamp with time zone" => PgType::TIMESTAMPTZ,
            "timestamp without time zone" => PgType::TIMESTAMP,
            "uuid" => PgType::UUID,
            _ => todo!("FromStr for {val:?}"),
        }))
    }
}

impl fmt::Display for Type {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use Type::*;
        match self {
            Array(inner) => fmt.write_fmt(format_args!("Vec<{inner}>")),
            Builtin(PgType::BOOL) => write!(fmt, "bool"),
            Builtin(PgType::BYTEA) => write!(fmt, "Vec<u8>"),
            Builtin(PgType::BYTEA_ARRAY) => write!(fmt, "Vec<Vec<u8>>"),
            Builtin(PgType::FLOAT4) => write!(fmt, "f32"),
            Builtin(PgType::FLOAT8) => write!(fmt, "f64"),
            Builtin(PgType::INET) => write!(fmt, "std::net::IpAddr"),
            Builtin(PgType::INT2) => write!(fmt, "i16"),
            Builtin(PgType::INT4) => write!(fmt, "i32"),
            Builtin(PgType::INT8) => write!(fmt, "i64"),
            Builtin(PgType::INTERVAL) => write!(fmt, "PgInterval"),
            Builtin(PgType::JSONB) => write!(fmt, "JsonB"),
            Builtin(PgType::TEXT) => write!(fmt, "String"),
            Builtin(PgType::TEXT_ARRAY) => write!(fmt, "Vec<String>"),
            Builtin(PgType::TIMESTAMP) => write!(fmt, "chrono::naive::NaiveDateTime"),
            Builtin(PgType::TIMESTAMPTZ) => write!(fmt, "chrono::DateTime<chrono::Utc>"),
            Builtin(PgType::UUID) => write!(fmt, "uuid::Uuid"),
            Builtin(inner) => todo!("fmt::Display for {inner:?}"),
            Composite(inner) => write!(fmt, "{}", AsUpperCamelCase(&inner.name)),
            Enum(inner) => write!(fmt, "{inner}"),
            Vector => write!(fmt, "pgvector::Vector"),
        }
    }
}

#[derive(Debug)]
pub enum TypeDefinition {
    Composite(Composite),
    Enum(Enum),
}

impl fmt::Display for TypeDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Composite(inner) => f.write_fmt(format_args!("{inner}")),
            Self::Enum(inner) => f.write_fmt(format_args!("{inner}")),
        }
    }
}

#[derive(Debug)]
pub struct Composite {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

impl Composite {
    pub(crate) async fn from_postgres(
        name: &str,
        client: &Client,
    ) -> Result<Self, tokio_postgres::Error> {
        let sql = "SELECT attname, atttypid
            FROM pg_type, pg_attribute
            WHERE typname = $1 AND pg_type.typrelid = pg_attribute.attrelid";

        let mut fields = Vec::new();
        for row in client.query(sql, &[&name]).await? {
            let name = row.get::<_, &str>(0);
            let r#type = Type::from_postgres_by_id(row.get(1), client).await?;
            fields.push((name.to_owned(), r#type));
        }

        Ok(Self {
            name: name.to_owned(),
            fields,
        })
    }
}

impl fmt::Display for Composite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("struct {} {{\n", AsUpperCamelCase(&self.name)))?;
        for (name, ty) in &self.fields {
            f.write_fmt(format_args!("    {}: {ty},\n", AsSnakeCase(&name)))?;
        }

        f.write_str("}\n")
    }
}

#[derive(Debug, PartialEq)]
pub struct Enum {
    name: String,
    variants: Vec<String>,
}

impl Enum {
    pub(crate) async fn from_postgres(
        name: &str,
        client: &Client,
    ) -> Result<Self, tokio_postgres::Error> {
        let mut new = Self {
            name: name.to_owned(),
            variants: Vec::new(),
        };

        let sql = r#"
            SELECT enumlabel
            FROM pg_catalog.pg_enum, pg_catalog.pg_type
            WHERE pg_type.typname = $1 AND pg_type.oid = enumtypid
            ORDER BY enumsortorder ASC
        "#;
        for row in client.query(sql, &[&name]).await? {
            let label = row.get::<_, &str>(0);
            new.variants.push(label.to_owned());
        }

        Ok(new)
    }
}

impl fmt::Display for Enum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("enum {} {{\n", AsUpperCamelCase(&self.name)))?;
        for variant in &self.variants {
            f.write_fmt(format_args!("    {},\n", AsUpperCamelCase(&variant)))?;
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
            Array(inner) => fmt.write_fmt(format_args!(
                "&{}{}{}[{}]",
                lt_prefix,
                lt_name,
                lt_suffix,
                TypeAsRef {
                    lifetime: *lifetime,
                    val: inner
                }
            )),
            Builtin(PgType::TEXT) => write!(fmt, "&{}{}{}str", lt_prefix, lt_name, lt_suffix),
            Builtin(PgType::TEXT_ARRAY) => {
                write!(fmt, "[&{}{}{}str]", lt_prefix, lt_name, lt_suffix)
            }
            Builtin(PgType::BYTEA) => write!(fmt, "&[u8]"),
            Builtin(PgType::BYTEA_ARRAY) => {
                write!(fmt, "&{}{}{}[u8]>", lt_prefix, lt_name, lt_suffix)
            }
            Builtin(PgType::JSONB) => write!(fmt, "&impl Serialize"),
            Builtin(_) => write!(fmt, "{val}"),
            Composite(inner) => write!(
                fmt,
                "&{}{}{}{}",
                lt_prefix,
                lt_name,
                lt_suffix,
                AsUpperCamelCase(&inner.name)
            ),
            Enum(inner) => fmt.write_fmt(format_args!("{inner}")),
            Vector => fmt.write_str("pgvector::Vector"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CompositeRef {
    pub(crate) name: String,
}

impl fmt::Display for CompositeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", &AsUpperCamelCase(&self.name)))
    }
}

#[derive(Debug, PartialEq)]
pub struct EnumRef {
    pub(crate) name: String,
}

impl fmt::Display for EnumRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", &AsUpperCamelCase(&self.name)))
    }
}
