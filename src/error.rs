use tree_sitter::QueryError;

#[derive(Debug)]
pub enum Error {
    QueryError(QueryError),
    IOError(std::io::Error),
}

impl From<QueryError> for Error {
    fn from(error: QueryError) -> Self {
        Error::QueryError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IOError(error)
    }
}
