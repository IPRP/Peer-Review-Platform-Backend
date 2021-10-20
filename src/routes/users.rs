use crate::db::models::*;
use crate::{db, IprpDB};
use rocket::data::{FromDataSimple, Outcome};
use rocket::http::{Cookie, Cookies, Status};
use rocket::{Data, Request};

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
    create_info: json::Json<RouteCreateStudent>,
) -> Result<json::Json<u64>, Status> {
    if user.username != "admin" {
        return Err(Status::Forbidden);
    }
    let hashed_password = crate::auth::crypto::hash_password(&create_info.0.password);
    let user = db::users::create_student(
        &*conn,
        create_info.0.username,
        create_info.0.firstname,
        create_info.0.lastname,
        hashed_password,
        create_info.0.unit,
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
    create_info: json::Json<RouteCreateTeacher>,
) -> Result<json::Json<u64>, Status> {
    if user.username != "admin" {
        return Err(Status::Forbidden);
    }
    let hashed_password = crate::auth::crypto::hash_password(&create_info.0.password);
    let user = db::users::create_teacher(
        &*conn,
        create_info.0.username,
        create_info.0.firstname,
        create_info.0.lastname,
        hashed_password,
    );
    return match user {
        Ok(user) => Ok(json::Json(user.id)),
        Err(_) => Err(Status::Conflict),
    };
}

// See: https://github.com/Keats/validator
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Debug, Validate, Deserialize)]
pub struct ValidatorTest {
    #[validate(length(min = 1))]
    detail: String,
    #[validate(custom = "not_empty")]
    #[serde(default)]
    details: Vec<String>,
}

fn not_empty(details: &Vec<String>) -> Result<(), ValidationError> {
    if details.is_empty() {
        return Err(ValidationError::new("details cannot be empty"));
    }
    Ok(())
}

// See: https://users.rust-lang.org/t/newbie-rust-rocket/35875/6
// And: https://github.com/SergioBenitez/Rocket/issues/1045#issuecomment-509036481
// TODO create trait with default implementation for this
impl FromDataSimple for ValidatorTest {
    type Error = ValidationErrors;

    fn from_data(_request: &Request, data: Data) -> Outcome<Self, Self::Error> {
        let json: serde_json::Result<ValidatorTest> = serde_json::from_reader(data.open());
        match json {
            Ok(value) => {
                if let Err(error) = value.validate() {
                    Outcome::Failure((Status::from_code(422).unwrap(), error))
                } else {
                    Outcome::Success(value)
                }
            }
            Err(_) => Outcome::Failure((Status::UnprocessableEntity, ValidationErrors::new())),
        }
    }
}

#[post("/validation", data = "<account>")]
pub fn validation_test(
    account: Result<ValidatorTest, ValidationErrors>,
) -> Result<json::Json<u64>, Status> {
    let value = account;

    return match value {
        Ok(_) => Ok(json::Json(42)),
        Err(_) => Err(Status::BadRequest),
    };
}
