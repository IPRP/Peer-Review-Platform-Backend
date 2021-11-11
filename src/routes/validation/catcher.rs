use crate::routes::models::ApiResponse;
use rocket::Request;
use std::borrow::Cow;
use validator::{ValidationError, ValidationErrors};

/// Return 422 errors as JSON with proper information
// Inspired by https://github.com/SergioBenitez/Rocket/issues/749#issuecomment-916292371
#[catch(422)]
pub fn unprocessable_entity(request: &Request) -> ApiResponse {
    let val_errors: &Option<ValidationErrors> = request.local_cache(|| None);
    match val_errors {
        Some(val_errors) => ApiResponse::unprocessable_entity(val_errors),
        _ => {
            let mut val_errors = ValidationErrors::new();
            let error = ValidationError {
                code: Cow::from("Unprocessable Entity".to_string()),
                message: None,
                params: Default::default(),
            };
            val_errors.add("general", error);
            ApiResponse::unprocessable_entity(&val_errors)
        }
    }
}
