use std::fmt::{Display, Formatter};

pub trait AppError: Display {
    fn source(&self) -> Option<&(dyn AppError + 'static)>;
    fn description(&self) -> String;
    fn fmt_error(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error>;
    fn fmt_stacktrace(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        stacktrace(self.source(), f);
        let _ = writeln!(f);
        let _ = self.fmt_error(f);
        Ok(())
    }
}

// impl Display for dyn AppError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         stacktrace(self.source(), f);
//         AppError::fmt_error(self, f);
//         Ok(())
//     }
// }

pub fn stacktrace(error: Option<&(dyn AppError + 'static)>, f: &mut Formatter<'_>) {
    if let Some(inner_error) = error {
        stacktrace(inner_error.source(), f);
        let _ = writeln!(f);
        let _ = AppError::fmt_error(inner_error, f);
    }
}
