pub enum QueryError {
    UnknownEnumValue,
}

pub type QueryResult<T> = Result<T, QueryError>;
