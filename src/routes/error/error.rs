use crate::utils::error::AppError;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Formatter};

#[derive(Debug)]
pub enum RouteErrorKind {
    BadRequest,
}

impl fmt::Display for RouteErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct RouteError {
    kind: RouteErrorKind,
    source: Option<Box<dyn AppError>>,
    error: Box<dyn Error + Send + Sync>,
}

impl RouteError {
    pub fn new<E>(kind: RouteErrorKind, error: E) -> RouteError
    where
        E: Into<Box<dyn Error + Send + Sync>>,
    {
        RouteError {
            kind,
            source: None,
            error: error.into(),
        }
    }

    pub fn new_with_source<E, S>(kind: RouteErrorKind, error: E, source: S) -> RouteError
    where
        E: Into<Box<dyn Error + Send + Sync>>,
        S: Into<Box<dyn AppError>>,
    {
        RouteError {
            kind,
            source: Some(source.into()),
            error: error.into(),
        }
    }
}

impl AppError for RouteError {
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

    fn fmt_error(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "DbError ({:?}): {:?}", self.kind, self.error)
    }
}

impl fmt::Display for RouteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_stacktrace(f)
    }
}
