#[derive(Debug, Clone, Copy)]
pub enum SqlType {
    Int,
    BigInt,
    SmallInt,
    Text,
    VarChar(usize),
    Bool,
    Timestamp,
    Date,
    Time,
    Float,
    Double,
    Decimal(u8, u8),
    Uuid,
    Json,
    JsonB,
}

pub trait ToSql: std::fmt::Display {
    fn sql_type() -> SqlType;
    fn to_sql(&self) -> String;
}

impl ToSql for i32 {
    fn sql_type() -> SqlType {
        SqlType::Int
    }
    fn to_sql(&self) -> String {
        self.to_string()
    }
}

impl ToSql for String {
    fn sql_type() -> SqlType {
        SqlType::Text
    }
    fn to_sql(&self) -> String {
        format!("'{}'", self)
    }
}

impl ToSql for bool {
    fn sql_type() -> SqlType {
        SqlType::Bool
    }
    fn to_sql(&self) -> String {
        if *self { "TRUE" } else { "FALSE" }.to_string()
    }
}

impl ToSql for uuid::Uuid {
    fn sql_type() -> SqlType {
        SqlType::Uuid
    }
    fn to_sql(&self) -> String {
        format!("'{}'", self)
    }
}
