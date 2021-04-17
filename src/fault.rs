use tree_sitter::QueryError;

#[derive(Debug)]
pub enum Fault {
    TSQuery(QueryError),
    Regular(FaultKind),
    Custom(String),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FaultKind {
    FileNotFound,
}

impl FaultKind {
    fn as_str(&self) -> &str {
        match *self {
            FaultKind::FileNotFound => "File not found",
        }
    }
}

impl From<QueryError> for Fault {
    fn from(error: QueryError) -> Self {
        Fault::TSQuery(error)
    }
}
