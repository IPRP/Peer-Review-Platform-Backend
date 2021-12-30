use std::fmt::{Display, Formatter};

pub trait AppError: Display {
    fn source(&self) -> Option<&(dyn AppError + 'static)>;
    fn description(&self) -> String;
}

pub fn fmt_source(error: Option<&(dyn AppError + 'static)>, f: &mut Formatter<'_>) {
    if let Some(inner_error) = error {
        let _ = inner_error.fmt(f);
        fmt_source(inner_error.source(), f);
    }
}
