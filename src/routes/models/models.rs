//! Structs used throughout routes

use crate::db::models::{Kind, NewCriterion};
use crate::routes::validation::SimpleValidation;
use chrono::Local;
use rocket::data::{FromDataSimple, Outcome};
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response};
use rocket::{response, Data};
use rocket_contrib::json::JsonValue;
use validator::{Validate, ValidationError, ValidationErrors};

// Submissions
#[derive(FromForm, Deserialize, Validate)]
pub struct RouteNewSubmission {
    #[validate(length(min = 1))]
    pub title: String,
    pub comment: String,
    #[serde(default)]
    pub attachments: NumberVec,
}

// TODO Define Macro for automatic code generation?
impl SimpleValidation for RouteNewSubmission {}

impl FromDataSimple for RouteNewSubmission {
    type Error = ValidationErrors;

    fn from_data(request: &Request, data: Data) -> Outcome<Self, Self::Error> {
        SimpleValidation::from_data(request, data)
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct RouteUpdateReview {
    pub feedback: String,
    #[serde(default)]
    #[validate]
    pub points: Vec<RouteUpdatePoints>,
}

impl SimpleValidation for RouteUpdateReview {}

impl FromDataSimple for RouteUpdateReview {
    type Error = ValidationErrors;

    fn from_data(request: &Request, data: Data) -> Outcome<Self, Self::Error> {
        SimpleValidation::from_data(request, data)
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct RouteUpdatePoints {
    pub id: u64,
    pub points: f64,
}

// Student & Teacher
#[derive(Serialize)]
pub struct RouteWorkshopResponse {
    pub(crate) id: u64,
    pub(crate) title: String,
}

// TODO: Struct Level validation
// See: https://github.com/Keats/validator/blob/master/validator_derive_tests/tests/schema.rs

#[derive(FromForm, Deserialize)]
pub struct RouteSearchStudent {
    pub(crate) all: bool,
    pub(crate) id: Option<u64>,
    pub(crate) firstname: Option<String>,
    pub(crate) lastname: Option<String>,
    pub(crate) group: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RouteCriterion {
    #[validate(length(min = 1))]
    title: String,
    content: String,
    #[serde(default = "route_criterion_default_weight")]
    #[validate(range(min = 0.0, max = 100.0))]
    weight: f64,
    #[serde(rename = "type")]
    kind: Kind,
}
// Pass default value to serde
// See: https://stackoverflow.com/a/65973982/12347616
const ROUTE_CRITERION_DEFAULT_WEIGHT: f64 = 1.0;
fn route_criterion_default_weight() -> f64 {
    ROUTE_CRITERION_DEFAULT_WEIGHT
}

impl From<RouteCriterion> for NewCriterion {
    fn from(item: RouteCriterion) -> Self {
        NewCriterion {
            title: item.title,
            content: item.content,
            weight: item.weight,
            kind: item.kind,
        }
    }
}

impl From<RouteCriterionVec> for Vec<NewCriterion> {
    fn from(items: RouteCriterionVec) -> Self {
        items
            .inner
            .into_iter()
            .map(|item| NewCriterion::from(item))
            .collect()
    }
}

// TODO convert to struct with named fields because of error
// #[derive(Validate)] can only be used on structs with named fields
#[derive(Deserialize, Validate)]
pub struct RouteCriterionVec {
    #[validate]
    pub(crate) inner: Vec<RouteCriterion>,
}

// Expected format is ISO 8601 combined date and time without timezone like `2007-04-05T14:30:30`
// In JS that would mean creating a Date like this `(new Date()).toISOString().split(".")[0]`
// https://github.com/serde-rs/json/issues/531#issuecomment-479738561
#[derive(Deserialize, Serialize, Validate)]
pub struct Date {
    pub(crate) inner: chrono::NaiveDateTime,
}

#[derive(Deserialize)]
pub struct NumberVec(pub(crate) Vec<u64>);

impl From<NumberVec> for Vec<u64> {
    fn from(items: NumberVec) -> Self {
        items.0
    }
}

impl Default for NumberVec {
    fn default() -> Self {
        NumberVec(Vec::new())
    }
}

#[derive(FromForm, Deserialize, Validate)]
pub struct RouteNewWorkshop {
    #[validate(length(min = 1))]
    pub(crate) title: String,
    pub(crate) content: String,
    #[serde(flatten, rename = "baum")]
    #[validate(custom = "deadline_check")]
    pub(crate) end: Date,
    pub(crate) anonymous: bool,
    pub(crate) teachers: NumberVec,
    pub(crate) students: NumberVec,
    #[validate]
    pub(crate) criteria: RouteCriterionVec,
    // Use default value
    // See: https://serde.rs/attr-default.html
    #[serde(default)]
    pub(crate) attachments: NumberVec,
}

impl SimpleValidation for RouteNewWorkshop {}

impl FromDataSimple for RouteNewWorkshop {
    type Error = ValidationErrors;

    fn from_data(request: &Request, data: Data) -> Outcome<Self, Self::Error> {
        SimpleValidation::from_data(request, data)
    }
}

#[derive(FromForm, Deserialize, Validate)]
pub struct RouteUpdateWorkshop {
    #[validate(length(min = 1))]
    pub(crate) title: String,
    pub(crate) content: String,
    #[validate(custom = "deadline_check")]
    pub(crate) end: Date,
    pub(crate) teachers: NumberVec,
    pub(crate) students: NumberVec,
    #[validate]
    pub(crate) criteria: RouteCriterionVec,
    #[serde(default)]
    pub(crate) attachments: NumberVec,
}

fn deadline_check(date: &Date) -> Result<(), ValidationError> {
    let current_date = Local::now().naive_local();
    if current_date > date.inner {
        return Err(ValidationError::new("Deadline cannot be in the past"));
    }
    Ok(())
}

impl SimpleValidation for RouteUpdateWorkshop {}

impl FromDataSimple for RouteUpdateWorkshop {
    type Error = ValidationErrors;

    fn from_data(request: &Request, data: Data) -> Outcome<Self, Self::Error> {
        SimpleValidation::from_data(request, data)
    }
}

// Users
#[derive(FromForm, Deserialize, Validate)]
pub struct RouteCreateStudent {
    #[validate(length(min = 1))]
    pub(crate) username: String,
    #[validate(length(min = 1))]
    pub(crate) firstname: String,
    #[validate(length(min = 1))]
    pub(crate) lastname: String,
    #[validate(length(min = 1))]
    pub(crate) password: String,
    #[serde(rename(deserialize = "group"))]
    pub(crate) unit: String,
}

impl SimpleValidation for RouteCreateStudent {}

impl FromDataSimple for RouteCreateStudent {
    type Error = ValidationErrors;

    fn from_data(request: &Request, data: Data) -> Outcome<Self, Self::Error> {
        SimpleValidation::from_data(request, data)
    }
}

#[derive(FromForm, Deserialize, Validate)]
pub struct RouteCreateTeacher {
    #[validate(length(min = 1))]
    pub(crate) username: String,
    #[validate(length(min = 1))]
    pub(crate) firstname: String,
    #[validate(length(min = 1))]
    pub(crate) lastname: String,
    #[validate(length(min = 1))]
    pub(crate) password: String,
}

impl SimpleValidation for RouteCreateTeacher {}

impl FromDataSimple for RouteCreateTeacher {
    type Error = ValidationErrors;

    fn from_data(request: &Request, data: Data) -> Outcome<Self, Self::Error> {
        SimpleValidation::from_data(request, data)
    }
}

// General

/// JSON response with custom status
// Based on: https://stackoverflow.com/a/54867136/12347616
#[derive(Debug)]
pub struct ApiResponse {
    json: JsonValue,
    status: Status,
}

impl ApiResponse {
    pub fn bad_request() -> Self {
        let json = json!({
            "ok": false
        });
        let status = Status::BadRequest;
        ApiResponse { json, status }
    }

    pub fn conflict() -> Self {
        let json = json!({
            "ok": false
        });
        let status = Status::Conflict;
        ApiResponse { json, status }
    }

    pub fn forbidden() -> Self {
        let json = json!({
            "ok": false
        });
        let status = Status::Forbidden;
        ApiResponse { json, status }
    }

    pub fn not_found() -> Self {
        let json = json!({
            "ok": false
        });
        let status = Status::NotFound;
        ApiResponse { json, status }
    }

    pub fn unprocessable_entity(validation_errors: &ValidationErrors) -> Self {
        let validation_errors = validation_errs_to_str_vec(&validation_errors);
        let json = json!(validation_errors);
        let status = Status::UnprocessableEntity;
        ApiResponse { json, status }
    }
}

impl<'r> Responder<'r> for ApiResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

// Source: https://blog.logrocket.com/json-input-validation-in-rust-web-services/
fn validation_errs_to_str_vec(ve: &ValidationErrors) -> Vec<String> {
    ve.field_errors()
        .iter()
        .map(|fe| {
            format!(
                "{}: errors: {}",
                fe.0,
                fe.1.iter()
                    .map(|ve| format!("{}: {:?}", ve.code, ve.params))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        })
        .collect()
}
