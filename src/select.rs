use std::borrow::Cow;
use std::marker::PhantomData;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Column<T> {
    pub name: &'static str,
    pub table: &'static str,
    pub phantom: std::marker::PhantomData<*const T>,
}

impl<T> Into<Cow<'static, str>> for &Column<T> {
    fn into(self) -> Cow<'static, str> {
        self.name.into()
    }
}

pub enum Either<T, E>
where
    E: Expr<Datatype = T>,
{
    Column {
        table: Cow<'static, str>,
        name: Cow<'static, str>,
        phantom: PhantomData<*const T>,
    },
    Expression {
        expr: E,
    },
}

impl<T> Into<Either<T, T>> for Column<T>
where
    T: Expr<Datatype = T>,
{
    fn into(self) -> Either<T, T> {
        Either::column(self.table.into(), self.name.into())
    }
}

impl<T, E> Either<T, E>
where
    E: Expr<Datatype = T>,
{
    pub fn column(table: Cow<'static, str>, name: Cow<'static, str>) -> Self {
        Self::Column {
            table,
            name,
            phantom: PhantomData::default(),
        }
    }

    pub fn expr(expr: E) -> Self {
        Self::Expression { expr }
    }
}

impl<T, E> Expr for Either<T, E>
where
    E: Expr<Datatype = T>,
{
    type Datatype = T;

    fn to_sql(&self) -> Cow<'static, str> {
        match self {
            Self::Column { table, name, .. } => format!("{}.{}", table, name).into(),
            Self::Expression { expr } => expr.to_sql(),
        }
    }
}

pub struct Query<E>
where
    E: Expr<Datatype = bool>,
{
    _prepare: bool,
    columns: Vec<Cow<'static, str>>,
    table: Cow<'static, str>,
    expression: Option<E>,
    _order_by: Option<Cow<'static, str>>,
}

impl Query<bool> {
    pub const NONE: Option<bool> = None;
}

impl<E> Query<E>
where
    E: Expr<Datatype = bool>,
{
    pub fn new(
        _prepare: bool,
        columns: Vec<Cow<'static, str>>,
        table: Cow<'static, str>,
        expression: Option<E>,
    ) -> Self {
        Self {
            _prepare,
            columns,
            table,
            expression,
            _order_by: None,
        }
    }

    pub fn to_string(&self) -> String {
        let columns = self.columns.iter().fold(String::new(), |mut acc, col| {
            if !acc.is_empty() {
                acc.push_str(", ");
            }
            acc.push_str(self.table.as_ref());
            acc.push('.');
            acc.push_str(col.as_ref());
            acc
        });
        let expr = if let Some(expr) = self.expression.as_ref() {
            expr.to_sql().into_owned()
        } else {
            String::new()
        };
        format!(
            "SELECT {columns} FROM {table} {_where} {expr}",
            columns = columns,
            table = self.table,
            _where = if self.expression.is_some() {
                "WHERE"
            } else {
                ""
            },
            expr = expr
        )
    }
}

pub trait Expr {
    type Datatype;

    fn to_sql(&self) -> Cow<'static, str>;
}

macro_rules! impl_base_types {
    ($t:ty, $to_sql:expr) => {
        impl Expr for $t {
            type Datatype = $t;
            fn to_sql(&self) -> Cow<'static, str> {
                $to_sql(self).into()
            }
        }
        impl Into<Either<$t, $t>> for $t {
            fn into(self) -> Either<$t, $t> {
                Either::expr(self)
            }
        }
    };
}

impl_base_types!(bool, |&b| if b { "TRUE" } else { "FALSE" });
impl_base_types!(i32, i32::to_string);
impl_base_types!(i64, i64::to_string);
impl_base_types!(String, |s: &String| {
    let mut tag = std::module_path!()
        .chars()
        .filter(char::is_ascii_alphabetic)
        .collect::<String>();
    while s.contains(&format!("${}$", &tag)) {
        tag.extend(
            std::module_path!()
                .chars()
                .filter(char::is_ascii_alphabetic),
        );
    }
    format!("${tag}${}${tag}$", s, tag = tag)
});

pub struct AndExpression<A, B>
where
    A: Expr<Datatype = bool>,
    B: Expr<Datatype = bool>,
{
    left: A,
    right: B,
}

impl<A, B> Expr for AndExpression<A, B>
where
    A: Expr<Datatype = bool>,
    B: Expr<Datatype = bool>,
{
    type Datatype = bool;
    fn to_sql(&self) -> Cow<'static, str> {
        format!("({}) AND ({})", self.left.to_sql(), self.right.to_sql()).into()
    }
}

pub struct OrExpression<A, B>
where
    A: Expr<Datatype = bool>,
    B: Expr<Datatype = bool>,
{
    left: A,
    right: B,
}

impl<A, B> Expr for OrExpression<A, B>
where
    A: Expr<Datatype = bool>,
    B: Expr<Datatype = bool>,
{
    type Datatype = bool;

    fn to_sql(&self) -> Cow<'static, str> {
        format!("({}) OR ({})", self.left.to_sql(), self.right.to_sql()).into()
    }
}

pub struct NotExpression<T>
where
    T: Expr<Datatype = bool>,
{
    expr: T,
}

impl<T> Expr for NotExpression<T>
where
    T: Expr<Datatype = bool>,
{
    type Datatype = bool;

    fn to_sql(&self) -> Cow<'static, str> {
        format!("NOT ({})", self.expr.to_sql()).into()
    }
}

pub struct ComparisonExpression<T, L, R>
where
    L: Expr<Datatype = T>,
    R: Expr<Datatype = T>,
{
    left: Either<T, L>,
    right: Either<T, R>,
    op: Operator,
    phantom: PhantomData<*const T>,
}

impl<T, L, R> Expr for ComparisonExpression<T, L, R>
where
    L: Expr<Datatype = T>,
    R: Expr<Datatype = T>,
{
    type Datatype = bool;

    fn to_sql(&self) -> Cow<'static, str> {
        format!(
            "({}) {} ({})",
            self.left.to_sql(),
            self.op,
            self.right.to_sql()
        )
        .into()
    }
}

pub enum Operator {
    LessThan,
    GreaterThan,
    LessThanOrEqualTo,
    GreaterThanOrEqualTo,
    Equal,
    NotEqual,
}

pub struct Where;

impl Where {
    pub fn and<L, R>(left: L, right: R) -> impl Expr<Datatype = bool>
    where
        L: Expr<Datatype = bool>,
        R: Expr<Datatype = bool>,
    {
        AndExpression { left, right }
    }

    pub fn or<L, R>(left: L, right: R) -> impl Expr<Datatype = bool>
    where
        L: Expr<Datatype = bool>,
        R: Expr<Datatype = bool>,
    {
        OrExpression { left, right }
    }

    pub fn not<T>(expr: T) -> impl Expr<Datatype = bool>
    where
        T: Expr<Datatype = bool>,
    {
        NotExpression { expr }
    }

    pub fn lt<T, L, R>(
        left: impl Into<Either<T, L>>,
        right: impl Into<Either<T, R>>,
    ) -> impl Expr<Datatype = bool>
    where
        L: Expr<Datatype = T>,
        R: Expr<Datatype = T>,
    {
        let left = left.into();
        let right = right.into();
        ComparisonExpression {
            left,
            right,
            op: Operator::LessThan,
            phantom: PhantomData::default(),
        }
    }

    pub fn gt<T, L, R>(
        left: impl Into<Either<T, L>>,
        right: impl Into<Either<T, R>>,
    ) -> impl Expr<Datatype = bool>
    where
        L: Expr<Datatype = T>,
        R: Expr<Datatype = T>,
    {
        let left = left.into();
        let right = right.into();
        ComparisonExpression {
            left,
            right,
            op: Operator::GreaterThan,
            phantom: PhantomData::default(),
        }
    }

    pub fn lte<T, L, R>(
        left: impl Into<Either<T, L>>,
        right: impl Into<Either<T, R>>,
    ) -> impl Expr<Datatype = bool>
    where
        L: Expr<Datatype = T>,
        R: Expr<Datatype = T>,
    {
        let left = left.into();
        let right = right.into();
        ComparisonExpression {
            left,
            right,
            op: Operator::LessThanOrEqualTo,
            phantom: PhantomData::default(),
        }
    }

    pub fn gte<T, L, R>(
        left: impl Into<Either<T, L>>,
        right: impl Into<Either<T, R>>,
    ) -> impl Expr<Datatype = bool>
    where
        L: Expr<Datatype = T>,
        R: Expr<Datatype = T>,
    {
        let left = left.into();
        let right = right.into();
        ComparisonExpression {
            left,
            right,
            op: Operator::GreaterThanOrEqualTo,
            phantom: PhantomData::default(),
        }
    }

    pub fn eq<T, L, R>(
        left: impl Into<Either<T, L>>,
        right: impl Into<Either<T, R>>,
    ) -> impl Expr<Datatype = bool>
    where
        L: Expr<Datatype = T>,
        R: Expr<Datatype = T>,
    {
        let left = left.into();
        let right = right.into();
        ComparisonExpression {
            left,
            right,
            op: Operator::Equal,
            phantom: PhantomData::default(),
        }
    }

    pub fn neq<T, L, R>(
        left: impl Into<Either<T, L>>,
        right: impl Into<Either<T, R>>,
    ) -> impl Expr<Datatype = bool>
    where
        L: Expr<Datatype = T>,
        R: Expr<Datatype = T>,
    {
        let left = left.into();
        let right = right.into();
        ComparisonExpression {
            left,
            right,
            op: Operator::NotEqual,
            phantom: PhantomData::default(),
        }
    }

    pub fn column<T>(table: Cow<'static, str>, name: Cow<'static, str>) -> Either<T, T>
    where
        T: Expr<Datatype = T>,
    {
        Either::Column {
            table,
            name,
            phantom: PhantomData::default(),
        }
    }

    pub fn expr<T, E>(expr: E) -> Either<T, E>
    where
        E: Expr<Datatype = T>,
    {
        Either::Expression { expr }
    }
}

impl std::fmt::Display for Operator {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::LessThan => {
                write!(fmt, "<")
            }
            Self::GreaterThan => {
                write!(fmt, ">")
            }
            Self::LessThanOrEqualTo => {
                write!(fmt, "<=")
            }
            Self::GreaterThanOrEqualTo => {
                write!(fmt, ">=")
            }
            Self::Equal => {
                write!(fmt, "=")
            }
            Self::NotEqual => {
                write!(fmt, "!=")
            }
        }
    }
}
