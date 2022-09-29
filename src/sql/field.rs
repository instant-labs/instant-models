use std::borrow::Cow;
use std::marker::PhantomData;

use crate::Table;

/// SQL column definition with the Rust type.
// TODO: add table type back-reference?
pub struct Field<Type, Table: crate::Table> {
    pub name: &'static str,
    // TODO: replace sea_query.
    pub iden: Table::IdenType,
    pub typ: PhantomData<Type>,
    pub table: PhantomData<Table>,
}

impl<Type, Table: crate::Table> Field<Type, Table> {
    pub const fn new(name: &'static str, iden: Table::IdenType) -> Self {
        Self {
            name,
            iden,
            typ: PhantomData::<Type>,
            table: PhantomData::<Table>,
        }
    }

    pub fn table() -> Table::IdenType {
        Table::table()
    }
}

// TODO: replace sea_query.
impl<Type, Table: crate::Table + 'static> sea_query::IntoIden for Field<Type, Table> {
    fn into_iden(self) -> sea_query::DynIden {
        self.iden.into_iden()
    }
}

/// Helper trait for converting tuples of fields into an iterator.
pub trait FieldList {
    type IntoIter: Iterator<Item = sea_query::DynIden>;

    fn into_iter(self) -> Self::IntoIter;
}

macro_rules! impl_field_list {
    ( $( $name:ident.$idx:tt )+ ) => {
        impl<$($name),+> FieldList for ($($name,)+)
        where $($name: sea_query::IntoIden,)+
        {
            type IntoIter = std::vec::IntoIter<sea_query::DynIden>;

            fn into_iter(self) -> Self::IntoIter {
                vec![$(self.$idx.into_iden(),)+].into_iter()
            }
        }
    };
}

// If you need to select more than 12 fields in a single query, open an issue.
impl_field_list!(A.0);
impl_field_list!(A.0 B.1);
impl_field_list!(A.0 B.1 C.2);
impl_field_list!(A.0 B.1 C.2 D.3);
impl_field_list!(A.0 B.1 C.2 D.3 E.4);
impl_field_list!(A.0 B.1 C.2 D.3 E.4 F.5);
impl_field_list!(A.0 B.1 C.2 D.3 E.4 F.5 G.6);
impl_field_list!(A.0 B.1 C.2 D.3 E.4 F.5 G.6 H.7);
impl_field_list!(A.0 B.1 C.2 D.3 E.4 F.5 G.6 H.7 I.8);
impl_field_list!(A.0 B.1 C.2 D.3 E.4 F.5 G.6 H.7 I.8 J.9);
impl_field_list!(A.0 B.1 C.2 D.3 E.4 F.5 G.6 H.7 I.8 J.9 K.10);
impl_field_list!(A.0 B.1 C.2 D.3 E.4 F.5 G.6 H.7 I.8 J.9 K.10 L.11);

/// Marker trait to indicate which types and fields can be compared.
pub trait Compatible<Type> {}

impl<Type, T1: Table, T2: Table> Compatible<Field<Type, T1>> for Field<Type, T2> {}

impl<Type, T1: Table, T2: Table> Compatible<Field<Type, T1>> for Field<Option<Type>, T2> {}

impl<Type, T1: Table, T2: Table> Compatible<Field<Option<Type>, T1>> for Field<Type, T2> {}

impl<Type, T: Table> Compatible<Field<Type, T>> for Type {}

impl<Type, T: Table> Compatible<Field<Type, T>> for Option<Type> {}

impl<Type, T: Table> Compatible<Field<Option<Type>, T>> for Type {}

macro_rules! impl_compatible {
    ( $t:ty | $( $s:ty ),+ ) => ($(
        impl<T: Table> Compatible<Field<$t, T>> for $s {}
        impl<T: Table> Compatible<Field<$t, T>> for Option<$s> {}
        impl<T: Table> Compatible<Field<Option<$t>, T>> for $s {}
        impl<T: Table> Compatible<Field<Option<$t>, T>> for Option<$s> {}

        impl<T1: Table, T2: Table> Compatible<Field<$t, T1>> for Field<$s, T2> {}
        impl<T1: Table, T2: Table> Compatible<Field<$t, T1>> for Field<Option<$s>, T2> {}
        impl<T1: Table, T2: Table> Compatible<Field<Option<$t>, T1>> for Field<$s, T2> {}
    )*)
}

impl_compatible!(String | &str, Cow<'static, str>);
