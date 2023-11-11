use std::borrow::Cow;
use std::fmt;

use heck::AsSnakeCase;

use crate::types::{Type, TypeAsRef};

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

pub struct NewValue<'a> {
    pub lifetime: Option<&'a str>,
    pub val: &'a Column,
}

impl fmt::Display for NewValue<'_> {
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
