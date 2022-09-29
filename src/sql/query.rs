use crate::{Combine, Compatible, Field, FieldList, Sources, Table};
use sea_query::{BinOper, IntoIden};
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
        let left_col = left.into().into_tbl_expr();
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
        Right: Compatible<Left> + IntoValueOrFieldRef,
    {
        let left_col = left.into().into_tbl_expr();
        let condition: sea_query::SimpleExpr = match right.into_value_or_field_ref() {
            ValueOrFieldRef::Value(value) => left_col.ne(value),
            ValueOrFieldRef::FieldRef(right_col) => {
                left_col.binary(BinOper::NotEqual, right_col.into_tbl_expr())
            }
        };
        Self {
            cond: sea_query::Cond::all().add(condition),
        }
    }

    pub fn gt<Left, Right>(left: Left, right: Right) -> Self
    where
        Left: Into<FieldRef>,
        Right: Compatible<Left> + IntoValueOrFieldRef,
    {
        let left_col = left.into().into_tbl_expr();
        let condition: sea_query::SimpleExpr = match right.into_value_or_field_ref() {
            ValueOrFieldRef::Value(value) => left_col.gt(value),
            ValueOrFieldRef::FieldRef(right_col) => {
                left_col.greater_than(right_col.into_tbl_expr())
            }
        };
        Self {
            cond: sea_query::Cond::all().add(condition),
        }
    }

    pub fn gte<Left, Right>(left: Left, right: Right) -> Self
    where
        Left: Into<FieldRef>,
        Right: Compatible<Left> + IntoValueOrFieldRef,
    {
        let left_col = left.into().into_tbl_expr();
        let condition: sea_query::SimpleExpr = match right.into_value_or_field_ref() {
            ValueOrFieldRef::Value(value) => left_col.gte(value),
            ValueOrFieldRef::FieldRef(right_col) => {
                left_col.greater_or_equal(right_col.into_tbl_expr())
            }
        };
        Self {
            cond: sea_query::Cond::all().add(condition),
        }
    }

    pub fn lt<Left, Right>(left: Left, right: Right) -> Self
    where
        Left: Into<FieldRef>,
        Right: Compatible<Left> + IntoValueOrFieldRef,
    {
        let left_col = left.into().into_tbl_expr();
        let condition: sea_query::SimpleExpr = match right.into_value_or_field_ref() {
            ValueOrFieldRef::Value(value) => left_col.lt(value),
            ValueOrFieldRef::FieldRef(right_col) => left_col.less_than(right_col.into_tbl_expr()),
        };
        Self {
            cond: sea_query::Cond::all().add(condition),
        }
    }

    pub fn lte<Left, Right>(left: Left, right: Right) -> Self
    where
        Left: Into<FieldRef>,
        Right: Compatible<Left> + IntoValueOrFieldRef,
    {
        let left_col = left.into().into_tbl_expr();
        let condition: sea_query::SimpleExpr = match right.into_value_or_field_ref() {
            ValueOrFieldRef::Value(value) => left_col.lte(value),
            ValueOrFieldRef::FieldRef(right_col) => {
                left_col.less_or_equal(right_col.into_tbl_expr())
            }
        };
        Self {
            cond: sea_query::Cond::all().add(condition),
        }
    }

    pub fn is<Left, Right>(left: Left, right: Right) -> Self
    where
        Left: Into<FieldRef>,
        Right: Compatible<Left> + Into<sea_query::Value>,
    {
        let left_col = left.into().into_tbl_expr();
        Self {
            cond: sea_query::Cond::all().add(left_col.is(right.into())),
        }
    }

    pub fn is_not<Left, Right>(left: Left, right: Right) -> Self
    where
        Left: Into<FieldRef>,
        Right: Compatible<Left> + Into<sea_query::Value>,
    {
        let left_col = left.into().into_tbl_expr();
        Self {
            cond: sea_query::Cond::all().add(left_col.is_not(right.into())),
        }
    }

    pub fn is_null<T>(col: T) -> Self
    where
        T: Into<FieldRef>,
    {
        let column = col.into().into_tbl_expr();
        Self {
            cond: sea_query::Cond::all().add(column.is_null()),
        }
    }

    pub fn is_not_null<T>(col: T) -> Self
    where
        T: Into<FieldRef>,
    {
        let column = col.into().into_tbl_expr();
        Self {
            cond: sea_query::Cond::all().add(column.is_not_null()),
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

impl FieldRef {
    pub fn into_tbl_expr(self) -> sea_query::Expr {
        sea_query::Expr::tbl(self.table, self.column)
    }
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
