use tree_sitter::QueryError;

#[derive(Debug)]
pub enum Fault {
    TSQuery(QueryError),
    FileNotFound(std::io::Error),
}

impl From<QueryError> for Fault {
    fn from(error: QueryError) -> Self {
        Fault::TSQuery(error)
    }
}

impl From<std::io::Error> for Fault {
    fn from(error: std::io::Error) -> Self {
        Fault::FileNotFound(error)
    }
}
