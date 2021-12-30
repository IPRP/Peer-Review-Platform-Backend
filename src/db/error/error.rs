use crate::utils::error::AppError;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Formatter};

// Inspired by https://doc.rust-lang.org/std/io/enum.ErrorKind.html
#[derive(Debug)]
pub enum DbErrorKind {
    NotFound,
    PastDeadline,
    Mismatch,
    CreateFailed,
    ReadFailed,
    UpdateFailed,
    DeleteFailed,
    TransactionFailed,
    EventCreateFailed,
}

impl fmt::Display for DbErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Inspired by https://learning-rust.github.io/docs/e7.custom_error_types.html
// And: https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
pub struct DbError {
    kind: DbErrorKind,
    source: Option<Box<dyn AppError>>,
    error: Box<dyn Error + Send + Sync>,
}

// See: https://doc.rust-lang.org/src/std/io/error.rs.html#407
impl DbError {
    pub fn new<E>(kind: DbErrorKind, error: E) -> DbError
    where
        E: Into<Box<dyn Error + Send + Sync>>,
    {
        DbError {
            kind,
            source: None,
            error: error.into(),
        }
    }

    pub fn new_with_source<E, S>(kind: DbErrorKind, error: E, source: S) -> DbError
    where
        E: Into<Box<dyn Error + Send + Sync>>,
        S: Into<Box<dyn AppError>>,
    {
        DbError {
            kind,
            source: Some(source.into()),
            error: error.into(),
        }
    }

    pub fn assign_and_rollback<T>(
        t_error: &mut Result<(), DbError>,
        error: DbError,
    ) -> Result<T, diesel::result::Error> {
        *t_error = Err(error);
        return Err(diesel::result::Error::RollbackTransaction);
    }
}

impl AppError for DbError {
    fn source(&self) -> Option<&(dyn AppError + 'static)> {
        // See: https://stackoverflow.com/a/65659930/12347616
        if let Some(source) = &self.source {
            Some(&**source)
        } else {
            None
        }
    }

    fn description(&self) -> String {
        self.error.to_string()
    }

    fn get_error_msg(&self) -> String {
        format!("DbError ({:?}): {:?}", self.kind, self.error)
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_stacktrace())
    }
}
