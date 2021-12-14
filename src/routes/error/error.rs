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
    error: Box<dyn Error + Send + Sync>,
}

impl RouteError {
    pub fn new<E>(kind: RouteErrorKind, error: E) -> RouteError
    where
        E: Into<Box<dyn Error + Send + Sync>>,
    {
        RouteError {
            kind,
            error: error.into(),
        }
    }
}

impl AppError for RouteError {
    fn description(&self) -> String {
        self.error.to_string()
    }
}

impl fmt::Display for RouteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DbError ({:?}): {:?}", self.kind, self.error)
    }
}
