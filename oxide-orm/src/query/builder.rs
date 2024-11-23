use std::{iter, marker::PhantomData};

use sqlx::{postgres::PgRow, FromRow};

use crate::{database::Error, Column, Database, Model, ModelColumns, ToSql};

pub struct OxideQueryBuilder<M: Model<C>, C: ModelColumns<Model = M>> {
    conditions: ConditionExpression,
    current_group: Option<ConditionExpression>,
    selected: Vec<String>,
    _marker: PhantomData<(M, C)>,
}

impl<M: Model<C>, C: ModelColumns<Model = M>> OxideQueryBuilder<M, C> {
    pub fn new() -> Self {
        Self {
            conditions: ConditionExpression::new(),
            current_group: None,
            selected: vec![],
            _marker: PhantomData,
        }
    }

    pub fn select<T, const N: usize>(mut self, columns: [Column<M, T>; N]) -> Self {
        self.selected
            .extend(columns.iter().map(|c| c.name.to_string()));
        self
    }

    pub fn select_one<T>(mut self, column: Column<M, T>) -> Self {
        self.selected.push(column.name.to_string());
        self
    }

    pub fn and_where<T: ToSql>(mut self, column: Column<M, T>, value: T) -> Self {
        let condition = Condition::Raw(format!("{} = {}", column.name, value.to_sql()));
        if let Some(group) = &mut self.current_group {
            group.expressions.push(condition);
        } else {
            self.conditions.expressions.push(condition);
        }
        self
    }

    pub fn or_where<T: ToSql>(mut self, column: Column<M, T>, value: T) -> Self {
        let mut or_group = ConditionExpression::new();
        or_group.expressions.push(Condition::Raw(format!(
            "{} = {}",
            column.name,
            value.to_sql()
        )));
        if let Some(group) = &mut self.current_group {
            group.expressions.push(Condition::Or(Box::new(or_group)));
        } else {
            self.conditions
                .expressions
                .push(Condition::Or(Box::new(or_group)));
        }
        self
    }

    pub fn and_group<F>(mut self, f: F) -> Self
    where
        F: FnOnce(OxideQueryBuilder<M, C>) -> OxideQueryBuilder<M, C>,
    {
        let builder = f(OxideQueryBuilder::new());
        let group_conditions = builder.conditions;
        if !group_conditions.expressions.is_empty() {
            if let Some(current) = &mut self.current_group {
                current
                    .expressions
                    .push(Condition::And(Box::new(group_conditions)));
            } else {
                self.conditions
                    .expressions
                    .push(Condition::And(Box::new(group_conditions)));
            }
        }
        self
    }

    pub fn or_group<F>(mut self, f: F) -> Self
    where
        F: FnOnce(OxideQueryBuilder<M, C>) -> OxideQueryBuilder<M, C>,
    {
        let builder = f(OxideQueryBuilder::new());
        let group_conditions = builder.conditions;
        if !group_conditions.expressions.is_empty() {
            if let Some(current) = &mut self.current_group {
                current
                    .expressions
                    .push(Condition::Or(Box::new(group_conditions)));
            } else {
                self.conditions
                    .expressions
                    .push(Condition::Or(Box::new(group_conditions)));
            }
        }
        self
    }

    pub fn build(&self) -> String {
        let columns = if self.selected.is_empty() {
            "*".to_string()
        } else {
            self.selected.join(", ")
        };

        let mut query = format!("SELECT {} FROM {}", columns, M::TABLE);

        let where_clause = self.conditions.build();
        if !where_clause.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&where_clause);
        }

        query
    }

    pub async fn fetch_all<T>(self, db: &Database) -> Result<Vec<T>, Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        db.query(self.build()).await
    }

    pub async fn fetch_one<T>(self, db: &Database) -> Result<T, Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        db.query_one(self.build()).await
    }

    pub async fn fetch_optional<T>(self, db: &Database) -> Result<Option<T>, Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        db.query_optional(self.build()).await
    }
}

#[derive(Debug, Clone)]
pub struct OxideInsertQueryBuilder<M: Model<C>, C: ModelColumns<Model = M>> {
    columns: Vec<String>,
    values: Vec<String>,
    _marker: PhantomData<(M, C)>,
}

impl<M: Model<C>, C: ModelColumns<Model = M>> OxideInsertQueryBuilder<M, C> {
    pub fn new() -> Self {
        Self {
            columns: vec![],
            values: vec![],
            _marker: PhantomData,
        }
    }

    pub fn value<T: ToSql>(mut self, column: Column<M, T>, value: T) -> Self {
        self.columns.push(column.name.to_string());
        self.values.push(value.to_sql());
        self
    }

    pub fn build(&self) -> String {
        let columns = self.columns.join(", ");
        let values = self.values.join(", ");
        format!("INSERT INTO {} ({}) VALUES ({})", M::TABLE, columns, values)
    }
}

#[derive(Clone)]
enum Condition {
    And(Box<ConditionExpression>),
    Or(Box<ConditionExpression>),
    Raw(String),
}

#[derive(Clone)]
struct ConditionExpression {
    expressions: Vec<Condition>,
}

impl ConditionExpression {
    fn new() -> Self {
        Self {
            expressions: vec![],
        }
    }

    fn build(&self) -> String {
        if self.expressions.is_empty() {
            return String::new();
        }

        let mut parts = Vec::new();
        let mut is_first = true;

        for condition in &self.expressions {
            match condition {
                Condition::And(expr) => {
                    if expr.expressions.is_empty() {
                        continue;
                    }
                    if !is_first {
                        parts.push("AND".to_string());
                    }
                    let expr_str = expr.build();
                    if expr.expressions.len() > 1 {
                        parts.push(format!("({})", expr_str));
                    } else {
                        parts.push(expr_str);
                    }
                }
                Condition::Or(expr) => {
                    if expr.expressions.is_empty() {
                        continue;
                    }
                    if !is_first {
                        parts.push("OR".to_string());
                    }
                    let expr_str = expr.build();
                    if expr.expressions.len() > 1 {
                        parts.push(format!("({})", expr_str));
                    } else {
                        parts.push(expr_str);
                    }
                }
                Condition::Raw(expr) => {
                    if !is_first {
                        parts.push("AND".to_string());
                    }
                    parts.push(expr.clone());
                }
            }
            is_first = false;
        }

        parts.join(" ")
    }
}
