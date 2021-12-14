use std::fmt::Display;

pub trait AppError: Display {
    fn description(&self) -> String;
}
