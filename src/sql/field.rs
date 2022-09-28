use std::borrow::Cow;
use std::marker::PhantomData;

/// SQL column definition with the Rust type.
// TODO: add table type back-reference?
pub struct Field<Type, Iden> {
    pub name: &'static str,
    // TODO: replace sea_query.
    pub iden: Iden,
    pub typ: PhantomData<Type>,
}

impl<Type, Iden> Field<Type, Iden> {
    pub const fn new(name: &'static str, iden: Iden) -> Self {
        Self {
            name,
            iden,
            typ: PhantomData::<Type>,
        }
    }
}

// TODO: replace sea_query.
impl<Type, Iden: sea_query::Iden + 'static> sea_query::IntoIden for Field<Type, Iden> {
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

impl<Type, Iden1, Iden2> Compatible<Field<Type, Iden1>> for Field<Type, Iden2> {}
impl<Type, Iden1, Iden2> Compatible<Field<Type, Iden1>> for Field<Option<Type>, Iden2> {}
impl<Type, Iden1, Iden2> Compatible<Field<Option<Type>, Iden1>> for Field<Type, Iden2> {}

impl<Type, Iden> Compatible<Field<Type, Iden>> for Type {}
impl<Type, Iden> Compatible<Field<Type, Iden>> for Option<Type> {}
impl<Type, Iden> Compatible<Field<Option<Type>, Iden>> for Type {}

macro_rules! impl_compatible {
    ( $t:ty | $( $s:ty ),+ ) => ($(
        impl<Iden> Compatible<Field<$t, Iden>> for $s {}
        impl<Iden> Compatible<Field<$t, Iden>> for Option<$s> {}
        impl<Iden> Compatible<Field<Option<$t>, Iden>> for $s {}
        impl<Iden> Compatible<Field<Option<$t>, Iden>> for Option<$s> {}

        impl<Iden1, Iden2> Compatible<Field<$t, Iden1>> for Field<$s, Iden2> {}
        impl<Iden1, Iden2> Compatible<Field<$t, Iden1>> for Field<Option<$s>, Iden2> {}
        impl<Iden1, Iden2> Compatible<Field<Option<$t>, Iden1>> for Field<$s, Iden2> {}
    )*)
}

impl_compatible!(String | &str, Cow<'static, str>);
