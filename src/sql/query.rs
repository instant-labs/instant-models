use crate::{Combine, Compatible, Field, FieldList, Sources, Table};
use sea_query::IntoIden;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

#[derive(Default)]
pub struct SqlQuery<T: ?Sized> {
    sources: PhantomData<T>,
    // TODO: replace sea_query.
    query: sea_query::SelectStatement,
}

// TODO: replace sea_query.
impl<T: Sources + ?Sized> SqlQuery<T> {
    pub fn new() -> SqlQuery<T> {
        let mut query = sea_query::SelectStatement::new();
        for table in T::tables() {
            query.from(table);
        }
        Self {
            query,
            sources: PhantomData::<T>,
        }
    }

    pub fn select<F, I>(mut self, columns: F) -> Self
    where
        F: FnOnce(T::SOURCES) -> I,
        I: FieldList,
    {
        self.query.columns(columns(T::sources()).into_iter());
        self
    }

    pub fn filter<F>(mut self, conditions: F) -> Self
    where
        F: FnOnce(T::SOURCES) -> Sql,
    {
        self.query.cond_where(conditions(T::sources()).cond);
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.query.limit(limit);
        self
    }

    pub fn from<O: Table + 'static>(mut self) -> SqlQuery<T::COMBINED>
    where
        T: Combine<O>,
    {
        use sea_query::IntoTableRef;
        self.query.from(O::table().into_table_ref());
        SqlQuery {
            sources: PhantomData::<T::COMBINED>,
            query: self.query,
        }
    }

    pub fn join<O: Table>(mut self) -> SqlQuery<T::COMBINED>
    where
        // TODO: restrict join to only tables with foreign keys.
        T: Combine<O>,
    {
        // TODO: join on foreign keys, or add them to a list and handle them on .finish()/whatever.
        // self.query.join(sea_query::JoinType::Join, T::table(), );
        SqlQuery {
            sources: PhantomData::<T::COMBINED>,
            query: self.query,
        }
    }
}

impl<T: ?Sized> Display for SqlQuery<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.query.to_string(sea_query::PostgresQueryBuilder)
        )
    }
}

#[cfg(feature = "postgres")]
impl<S: Sources + ?Sized> SqlQuery<S> {
    /// Executes a query, returning the resulting rows.
    pub fn fetch(
        self,
        client: &mut postgres::Client,
        params: &[&(dyn postgres_types::ToSql + Sync)],
    ) -> std::result::Result<Vec<postgres::Row>, postgres::Error> {
        client.query(&self.to_string(), params)
    }
}

/// SQL condition for e.g. WHERE, ON, HAVING clauses.
///
/// Can be composed using bitwise operators `&` for AND, `|` for OR.
pub struct Sql {
    // TODO: replace sea_query.
    cond: sea_query::Cond,
}

impl Sql {
    pub fn eq<Left, Right>(left: Left, right: Right) -> Self
    where
        Left: Into<FieldRef>,
        Right: Compatible<Left> + IntoValueOrFieldRef,
    {
        let FieldRef { table, column } = left.into();
        let left_col = sea_query::Expr::tbl(table, column);
        let condition: sea_query::SimpleExpr = match right.into_value_or_field_ref() {
            ValueOrFieldRef::Value(value) => left_col.eq(value),
            ValueOrFieldRef::FieldRef(right_col) => {
                left_col.equals(right_col.table, right_col.column)
            }
        };
        Self {
            cond: sea_query::Cond::all().add(condition),
        }
    }

    pub fn ne<Left, Right>(left: Left, right: Right) -> Self
    where
        Left: Into<FieldRef>,
        Right: Compatible<Left> + Into<sea_query::Value>,
    {
        let left: FieldRef = left.into();
        Self {
            cond: sea_query::Cond::all()
                .add(sea_query::Expr::tbl(left.table, left.column).ne(right.into())),
        }
    }

    pub fn is_null<T>(col: T) -> Self
    where
        T: Into<FieldRef>,
    {
        let FieldRef { table, column } = col.into();
        Self {
            cond: sea_query::Cond::all().add(sea_query::Expr::tbl(table, column).is_null()),
        }
    }

    pub fn is_not_null<T>(col: T) -> Self
    where
        T: Into<FieldRef>,
    {
        let FieldRef { table, column } = col.into();
        Self {
            cond: sea_query::Cond::all().add(sea_query::Expr::tbl(table, column).is_not_null()),
        }
    }

    // TODO: port rest of conditions.
}

impl std::ops::BitAnd for Sql {
    type Output = Sql;

    fn bitand(self, rhs: Self) -> Self::Output {
        Sql {
            cond: sea_query::Cond::all().add(self.cond).add(rhs.cond),
        }
    }
}

impl std::ops::BitOr for Sql {
    type Output = Sql;

    fn bitor(self, rhs: Self) -> Self::Output {
        Sql {
            cond: sea_query::Cond::any().add(self.cond).add(rhs.cond),
        }
    }
}

/// Field reference with explicit table and column identifiers.
pub struct FieldRef {
    table: sea_query::DynIden,
    column: sea_query::DynIden,
}

impl<Type, Table: crate::Table + 'static> From<Field<Type, Table>> for FieldRef {
    fn from(field: Field<Type, Table>) -> FieldRef {
        FieldRef {
            table: Table::table().into_iden(),
            column: field.iden.into_iden(),
        }
    }
}

pub enum ValueOrFieldRef {
    Value(sea_query::Value),
    FieldRef(FieldRef),
}

pub trait IntoValueOrFieldRef {
    fn into_value_or_field_ref(self) -> ValueOrFieldRef;
}

impl<Type, Table: crate::Table + 'static> IntoValueOrFieldRef for Field<Type, Table> {
    fn into_value_or_field_ref(self) -> ValueOrFieldRef {
        ValueOrFieldRef::FieldRef(FieldRef {
            table: Table::table().into_iden(),
            column: self.iden.into_iden(),
        })
    }
}

impl<V> IntoValueOrFieldRef for V
where
    V: Into<sea_query::Value>,
{
    fn into_value_or_field_ref(self) -> ValueOrFieldRef {
        ValueOrFieldRef::Value(self.into())
    }
}
