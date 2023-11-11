use std::borrow::Cow;

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

impl std::fmt::Display for NewValue<'_> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
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
