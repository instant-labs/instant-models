use crate::{Combine, Sources, Table};
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

    pub fn select<F, C, I>(mut self, columns: F) -> Self
    where
        F: FnOnce(T::SOURCES) -> I,
        C: sea_query::IntoColumnRef,
        I: IntoIterator<Item = C>,
    {
        self.query.columns(columns(T::sources()));
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

    pub fn from<O: Table>(mut self) -> SqlQuery<T::COMBINED>
    where
        T: Combine<O>,
    {
        self.query.from(O::table());
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
    pub fn eq<T, V>(col: T, value: V) -> Self
    where
        T: sea_query::IntoColumnRef,
        V: Into<sea_query::Value>,
    {
        Self {
            cond: sea_query::Cond::all().add(sea_query::Expr::col(col).eq(value)),
        }
    }

    pub fn equals<T, U, V>(left: T, table: U, right: V) -> Self
    where
        T: sea_query::IntoColumnRef,
        U: sea_query::IntoIden,
        V: sea_query::IntoIden,
    {
        Self {
            cond: sea_query::Cond::all().add(sea_query::Expr::col(left).equals(table, right)),
        }
    }

    pub fn ne<T, V>(col: T, value: V) -> Self
    where
        T: sea_query::IntoColumnRef,
        V: Into<sea_query::Value>,
    {
        Self {
            cond: sea_query::Cond::all().add(sea_query::Expr::col(col).ne(value)),
        }
    }

    pub fn is_null<T>(col: T) -> Self
    where
        T: sea_query::IntoColumnRef,
    {
        Self {
            cond: sea_query::Cond::all().add(sea_query::Expr::col(col).is_null()),
        }
    }

    pub fn is_not_null<T>(col: T) -> Self
    where
        T: sea_query::IntoColumnRef,
    {
        Self {
            cond: sea_query::Cond::all().add(sea_query::Expr::col(col).is_not_null()),
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
