//! Structs used throughout routes

use crate::db::models::{Kind, NewCriterion};
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response;
use rocket::response::{Responder, Response};
use rocket_contrib::json::JsonValue;
use validator::ValidationErrors;

// Submissions
#[derive(FromForm, Deserialize)]
pub struct RouteNewSubmission {
    pub title: String,
    pub comment: String,
    pub attachments: NumberVec,
}

#[derive(Serialize, Deserialize)]
pub struct RouteUpdateReview {
    pub feedback: String,
    pub points: Vec<RouteUpdatePoints>,
}

#[derive(Serialize, Deserialize)]
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

#[derive(FromForm, Deserialize)]
pub struct RouteSearchStudent {
    pub(crate) all: bool,
    pub(crate) id: Option<u64>,
    pub(crate) firstname: Option<String>,
    pub(crate) lastname: Option<String>,
    pub(crate) group: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RouteCriterion {
    title: String,
    content: String,
    weight: f64,
    #[serde(rename = "type")]
    kind: Kind,
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
            .0
            .into_iter()
            .map(|item| NewCriterion::from(item))
            .collect()
    }
}

#[derive(Deserialize)]
pub struct RouteCriterionVec(pub(crate) Vec<RouteCriterion>);

// Expected format is ISO 8601 combined date and time without timezone like `2007-04-05T14:30:30`
// In JS that would mean creating a Date like this `(new Date()).toISOString().split(".")[0]`
// https://github.com/serde-rs/json/issues/531#issuecomment-479738561
#[derive(Deserialize)]
pub struct Date(pub(crate) chrono::NaiveDateTime);

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

#[derive(FromForm, Deserialize)]
pub struct RouteNewWorkshop {
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) end: Date,
    pub(crate) anonymous: bool,
    pub(crate) teachers: NumberVec,
    pub(crate) students: NumberVec,
    pub(crate) criteria: RouteCriterionVec,
    // Use default value
    // See: https://serde.rs/attr-default.html
    #[serde(default)]
    pub(crate) attachments: NumberVec,
}

#[derive(FromForm, Deserialize)]
pub struct RouteUpdateWorkshop {
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) end: Date,
    pub(crate) teachers: NumberVec,
    pub(crate) students: NumberVec,
    pub(crate) criteria: RouteCriterionVec,
    #[serde(default)]
    pub(crate) attachments: NumberVec,
}

// Users
#[derive(FromForm, Deserialize)]
pub struct RouteCreateStudent {
    pub(crate) username: String,
    pub(crate) firstname: String,
    pub(crate) lastname: String,
    pub(crate) password: String,
    #[serde(rename(deserialize = "group"))]
    pub(crate) unit: String,
}

#[derive(FromForm, Deserialize)]
pub struct RouteCreateTeacher {
    pub(crate) username: String,
    pub(crate) firstname: String,
    pub(crate) lastname: String,
    pub(crate) password: String,
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
