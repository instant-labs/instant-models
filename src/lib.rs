//use postgres_types::Json;
//use time::{OffsetDateTime, PrimitiveDateTime};
use heck::{AsSnakeCase, AsUpperCamelCase};
use indexmap::IndexMap;
use std::borrow::Cow;
pub use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct ForeignKey {
    to_table: Cow<'static, str>,
    columns: Vec<Cow<'static, str>>,
}

#[derive(Debug, PartialEq)]
pub struct Column {
    pub name: Cow<'static, str>,
    pub r#type: Type,
    pub null: bool,
    pub primary_key: bool,
    pub foreign_key: Option<ForeignKey>,
    pub unique: bool,
    pub default: Option<Cow<'static, str>>,
    pub type_def: Option<Cow<'static, str>>,
}

impl Column {
    pub fn new(name: Cow<'static, str>, r#type: Type) -> Self {
        Self {
            name,
            r#type,
            null: false,
            primary_key: false,
            foreign_key: None,
            unique: false,
            default: None,
            type_def: None,
        }
    }

    pub fn set_null(mut self, value: bool) -> Self {
        self.null = value;
        self
    }

    pub fn set_primary_key(mut self, value: bool) -> Self {
        self.primary_key = value;
        self
    }

    pub fn set_foreign_key(mut self, value: Option<ForeignKey>) -> Self {
        self.foreign_key = value;
        self
    }

    pub fn set_unique(mut self, value: bool) -> Self {
        self.unique = value;
        self
    }

    pub fn set_default(mut self, value: Option<Cow<'static, str>>) -> Self {
        self.default = value;
        self
    }

    pub fn set_type_def(mut self, value: Option<Cow<'static, str>>) -> Self {
        self.type_def = value;
        self
    }
}

impl std::fmt::Display for Column {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.null {
            write!(fmt, "{}: Option<{}>,", AsSnakeCase(&self.name), self.r#type)
        } else {
            write!(fmt, "{}: {},", AsSnakeCase(&self.name), self.r#type)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Constraint {
    ForeignKey {
        name: Cow<'static, str>,
        columns: Cow<'static, [Cow<'static, str>]>,
        ref_table: Cow<'static, str>,
        ref_columns: Cow<'static, [Cow<'static, str>]>,
    },
    PrimaryKey {
        name: Cow<'static, str>,
        columns: Vec<Cow<'static, str>>,
    },
}

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

    pub fn build(self) -> String {
        format!("{}", self)
    }
}

impl std::fmt::Display for StructBuilder {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let columns = self.columns.values().fold(String::new(), |mut acc, col| {
            acc.push_str(&format!("    {}", col));
            acc.push('\n');
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

#[derive(Debug, PartialEq)]
pub enum Type {
    Builtin { inner: postgres_types::Type },
    Composite { inner: StructBuilder },
}

impl FromStr for Type {
    type Err = ();
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
