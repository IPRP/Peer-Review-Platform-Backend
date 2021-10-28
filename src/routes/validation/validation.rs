use rocket::data::Outcome;
use rocket::http::Status;
use rocket::{Data, Request};
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use validator::{Validate, ValidationError, ValidationErrors};

/// Trait `SimpleValidation` provides parsing & validation for easy usage with `FromDataSimple`.
pub trait SimpleValidation
where
    Self: std::marker::Sized + DeserializeOwned + Validate,
{
    // See: https://users.rust-lang.org/t/newbie-rust-rocket/35875/6
    // And: https://github.com/SergioBenitez/Rocket/issues/1045#issuecomment-509036481
    fn from_data<'a>(request: &Request, data: Data) -> Outcome<Self, ValidationErrors> {
        let json: serde_json::Result<Self> = serde_json::from_reader(data.open());
        match json {
            Ok(value) => {
                if let Err(val_errors) = value.validate() {
                    request.local_cache(|| Some(val_errors.clone()));
                    Outcome::Failure((Status::UnprocessableEntity, val_errors))
                } else {
                    Outcome::Success(value)
                }
            }
            Err(parse_error) => {
                let mut val_errors = ValidationErrors::new();
                let error = ValidationError {
                    code: Cow::from(parse_error.to_string()),
                    message: None,
                    params: Default::default(),
                };
                val_errors.add("general", error);
                request.local_cache(|| Some(val_errors.clone()));
                Outcome::Failure((Status::UnprocessableEntity, val_errors))
            }
        }
    }
}
