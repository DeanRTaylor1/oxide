use std::marker::PhantomData;

use crate::{Column, Database, Model, ModelColumns, ToSql};

pub struct OxideQueryBuilder<M: Model<C>, C: ModelColumns<Model = M>> {
    conditions: Vec<String>,
    selected: Vec<String>,
    _marker: PhantomData<(M, C)>,
}

impl<M: Model<C>, C: ModelColumns<Model = M>> OxideQueryBuilder<M, C> {
    pub fn new() -> Self {
        Self {
            conditions: vec![],
            selected: vec![],
            _marker: PhantomData,
        }
    }

    pub fn select<T: ToSql>(mut self, column: Column<M, T>) -> Self {
        self.selected.push(column.name.to_string());
        self
    }

    pub fn where_eq<T: ToSql>(mut self, column: Column<M, T>, value: T) -> Self {
        self.conditions
            .push(format!("{} = {}", column.name, value.to_sql()));
        self
    }

    pub fn build(&self) -> String {
        let columns = if self.selected.is_empty() {
            "*".to_string()
        } else {
            self.selected.join(", ")
        };

        let mut query = format!("SELECT {} FROM {}", columns, M::TABLE);
        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.conditions.join(" AND "));
        }
        query
    }
}
