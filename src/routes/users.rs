use crate::db::models::*;
use crate::{db, IprpDB};
use rocket::http::{Cookie, Cookies, Status};

use crate::routes::models::{RouteCreateStudent, RouteCreateTeacher};
use rocket_contrib::json;
use rocket_contrib::json::{Json, JsonValue};

/// Use Basic Auth header to trigger this.
/// Using cookies will resend the cookie.
#[post("/login")]
pub fn login(user: User, mut cookies: Cookies) -> Json<JsonValue> {
    cookies.add_private(Cookie::new("user_id", user.id.to_string()));
    let role = user.role.to_string();
    Json(json!({ "id": user.id, "role": role }))
}

/// Removes set cookie.
#[post("/logout")]
pub fn logout(_user: User, mut cookies: Cookies) -> Status {
    cookies.remove_private(Cookie::named("user_id"));
    Status::Ok
}

/// Create new student account.
/// Only accessible with "admin" account.
#[post("/users/student", format = "json", data = "<create_info>")]
pub fn create_student(
    user: User,
    conn: IprpDB,
    create_info: RouteCreateStudent,
) -> Result<json::Json<u64>, Status> {
    if user.username != "admin" {
        return Err(Status::Forbidden);
    }
    let hashed_password = crate::auth::crypto::hash_password(&create_info.password);
    let user = db::users::create_student(
        &*conn,
        create_info.username,
        create_info.firstname,
        create_info.lastname,
        hashed_password,
        create_info.unit,
    );
    return match user {
        Ok(user) => Ok(json::Json(user.id)),
        Err(_) => Err(Status::Conflict),
    };
}

/// Create new teacher account.
/// Only accessible with "admin" account.
#[post("/users/teacher", format = "json", data = "<create_info>")]
pub fn create_teacher(
    user: User,
    conn: IprpDB,
    create_info: RouteCreateTeacher,
) -> Result<json::Json<u64>, Status> {
    if user.username != "admin" {
        return Err(Status::Forbidden);
    }
    let hashed_password = crate::auth::crypto::hash_password(&create_info.password);
    let user = db::users::create_teacher(
        &*conn,
        create_info.username,
        create_info.firstname,
        create_info.lastname,
        hashed_password,
    );
    return match user {
        Ok(user) => Ok(json::Json(user.id)),
        Err(_) => Err(Status::Conflict),
    };
}

/*// See: https://github.com/Keats/validator
use crate::routes::validation::SimpleValidation;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Debug, Validate, Deserialize)]
pub struct ValidatorTest {
    #[validate(length(min = 1))]
    detail: String,
    detail2: u64,
    #[validate(custom = "not_empty")]
    #[serde(default)]
    details: Vec<String>,
}

fn not_empty(details: &Vec<String>) -> Result<(), ValidationError> {
    if details.is_empty() {
        return Err(ValidationError::new("cannot be empty"));
    }
    Ok(())
}

impl SimpleValidation for ValidatorTest {}

impl FromDataSimple for ValidatorTest {
    type Error = ValidationErrors;

    fn from_data(request: &Request, data: Data) -> Outcome<Self, Self::Error> {
        SimpleValidation::from_data(request, data)
    }
}

#[post("/validation", data = "<account>")]
pub fn validation_test(
    account: Result<ValidatorTest, ValidationErrors>,
) -> Result<json::Json<u64>, ApiResponse> {
    let value = account;

    // let ok = match value {
    //     Ok(_) => Ok(json::Json(42)),
    //     Err(val_errors) => {
    //         //let errors = validation_errs_to_str_vec(&val_errors);
    //         //let errors: Vec<String> = Vec::from(SimpleValidationErrors(val_errors));
    //         let errors = val_errors;
    //         return Err(ApiResponse::unprocessable_entity(errors));
    //     }
    // };
    // let do_something = "hello world";
    // ok

    // match value {
    //     Ok(_) => Ok(json::Json(42)),
    //     Err(val_errors) => Err(ApiResponse::unprocessable_entity(val_errors)),
    // }

    value
        .map(|_| json::Json(42))
        .map_err(|val_errors| ApiResponse::unprocessable_entity(&val_errors))
}

#[post("/validation2", data = "<account>")]
pub fn validation_test2(account: ValidatorTest) -> json::Json<u64> {
    let _ = account;
    json::Json(42)
}

// See: https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validator_test_ok() {
        let vt = ValidatorTest {
            detail: "abc".to_string(),
            detail2: 42,
            details: vec!["Hello".to_string(), "World".to_string()],
        };
        assert!(vt.validate().is_ok());
    }

    #[test]
    fn validator_test_not_ok() {
        let vt = ValidatorTest {
            detail: "".to_string(),
            detail2: 42,
            details: vec!["Hello".to_string(), "World".to_string()],
        };
        assert!(vt.validate().is_err());
        let vt = ValidatorTest {
            detail: "abc".to_string(),
            detail2: 42,
            details: vec![],
        };
        assert!(vt.validate().is_err());
    }
}*/
