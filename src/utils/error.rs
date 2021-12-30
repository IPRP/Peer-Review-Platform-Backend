use std::fmt::{Display, Formatter};

pub trait AppError: Display {
    fn source(&self) -> Option<&(dyn AppError + 'static)>;
    fn description(&self) -> String;

    fn get_error_msg(&self) -> String;
    fn get_stacktrace(&self) -> String {
        let mut buffer = String::new();
        inner_stacktrace_processing(self.source(), &mut buffer);
        buffer.push_str(&*self.get_error_msg());
        buffer
    }

    fn print_error(&self) {
        println!("{}", self.get_error_msg());
    }

    fn print_stacktrace(&self) {
        println!("{}", self.get_stacktrace());
    }
}

pub fn inner_stacktrace_processing(error: Option<&(dyn AppError + 'static)>, buffer: &mut String) {
    if let Some(inner_error) = error {
        inner_stacktrace_processing(inner_error.source(), buffer);
        buffer.push_str(&*inner_error.get_error_msg());
        buffer.push_str("\n");
    }
}

// impl Display for dyn AppError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         stacktrace(self.source(), f);
//         AppError::fmt_error(self, f);
//         Ok(())
//     }
// }

// pub fn stacktrace(error: Option<&(dyn AppError + 'static)>, f: &mut Formatter<'_>) {
//     if let Some(inner_error) = error {
//         stacktrace(inner_error.source(), f);
//         let _ = writeln!(f);
//         let _ = AppError::fmt_error(inner_error, f);
//     }
// }
