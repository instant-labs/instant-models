use std::fmt;
use std::sync::Arc;

use heck::AsSnakeCase;

use crate::types::{Type, TypeAsRef};

#[derive(Debug, PartialEq)]
pub struct Column {
    pub name: Arc<str>,
    pub r#type: Type,
    pub null: bool,
    pub primary_key: bool,
    pub foreign_key: Option<ForeignKey>,
    pub unique: bool,
    pub default: Option<String>,
    pub type_def: Option<String>,
}

impl Column {
    pub fn new(name: Arc<str>, r#type: Type) -> Self {
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

    pub fn new_field(&self) -> NewField {
        NewField {
            lifetime: Some("a"),
            val: self,
        }
    }
}

impl fmt::Display for Column {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.null {
            write!(fmt, "{}: Option<{}>", AsSnakeCase(&self.name), self.r#type)
        } else {
            write!(fmt, "{}: {}", AsSnakeCase(&self.name), self.r#type)
        }
    }
}

pub struct NewField<'a> {
    pub lifetime: Option<&'a str>,
    pub val: &'a Column,
}

impl fmt::Display for NewField<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.val.null && self.val.default.is_some() {
            panic!(
                "Column `{}` is both NULL and takes a default value `{}`",
                self.val.name,
                self.val.default.as_ref().unwrap()
            );
        }
        if self.val.null || self.val.default.is_some() {
            write!(
                fmt,
                "{}: Option<{}>",
                AsSnakeCase(&self.val.name),
                TypeAsRef {
                    lifetime: self.lifetime,
                    val: &self.val.r#type
                }
            )
        } else {
            write!(
                fmt,
                "{}: {}",
                AsSnakeCase(&self.val.name),
                TypeAsRef {
                    lifetime: self.lifetime,
                    val: &self.val.r#type
                }
            )
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Constraint {
    ForeignKey {
        name: String,
        columns: Vec<String>,
        ref_table: String,
        ref_columns: Vec<String>,
    },
    PrimaryKey {
        name: Vec<String>,
        columns: Vec<String>,
    },
}

#[derive(Debug, PartialEq)]
pub struct ForeignKey {
    to_table: String,
    columns: Vec<String>,
}
