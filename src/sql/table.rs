use crate::SqlQuery;

/// A SQL table with field identifiers.
pub trait Table {
    type FieldsType;
    const FIELDS: Self::FieldsType;

    fn query() -> SqlQuery<Self> {
        SqlQuery::new()
    }

    /// Returns a reference to the table identifier.
    // TODO: replace sea_query.
    fn table() -> sea_query::TableRef;

    /// Returns a struct with the SQL column identifiers.
    fn fields() -> Self::FieldsType {
        Self::FIELDS
    }
}

/// Represents one or more SQL [Tables](Table).
pub trait Sources {
    type SOURCES;

    /// Table field definitions. Returns a tuple if more than one table is referenced.
    fn sources() -> Self::SOURCES;

    /// List of all table identifiers.
    fn tables() -> Vec<sea_query::TableRef>;
}

impl<T: Table + ?Sized> Sources for T {
    type SOURCES = T::FieldsType;

    fn sources() -> Self::SOURCES {
        T::fields()
    }

    fn tables() -> Vec<sea_query::TableRef> {
        use sea_query::IntoTableRef;
        vec![T::table().into_table_ref()]
    }
}

/// Helper trait for combining tuples of SQL tables.
///
/// E.g.
/// - A + B => (A,B).
/// - (A,B) + C => (A,B,C).
pub trait Combine<O> {
    type COMBINED;
}

impl<A: Table, B: Table> Combine<B> for A {
    type COMBINED = (A, B);
}

macro_rules! impl_sources_tuple {
    ( $( $name:ident )+ ) => {
        impl<$($name: Table),+> Sources for ($($name,)+)
        {
            type SOURCES = ($($name::FieldsType,)+);

            fn sources() -> Self::SOURCES {
                ($($name::fields(),)+)
            }

            fn tables() -> Vec<sea_query::TableRef> {
                use sea_query::IntoTableRef;
                vec![$($name::table().into_table_ref(),)+]
            }
        }
    };
    ( $( $name:ident )+, $joinable:expr ) => {
        impl_sources_tuple!($($name)+);

        impl<Z: Table, $($name: Table),+> Combine<Z> for ($($name,)+) {
            type COMBINED = ($($name,)+ Z);
        }
    };
}

// Implement `Sources` for tuples of tables: (A,B), (A,B,C), etc.
// If you want to join more than ten tables in a single query, open an issue.
impl_sources_tuple! { A B, true }
impl_sources_tuple! { A B C, true }
impl_sources_tuple! { A B C D, true }
impl_sources_tuple! { A B C D E, true }
impl_sources_tuple! { A B C D E F, true }
impl_sources_tuple! { A B C D E F G, true }
impl_sources_tuple! { A B C D E F G H, true }
impl_sources_tuple! { A B C D E F G H I, true }
impl_sources_tuple! { A B C D E F G H I J }
