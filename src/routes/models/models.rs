//! Structs used throughout routes

use crate::db::error::DbError;
use crate::db::models::{Kind, NewCriterion};
use crate::routes::validation::SimpleValidation;
use backend_macro_derive::SimpleValidation;
use chrono::Local;
use rocket::data::{FromDataSimple, Outcome};
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response};
use rocket::{response, Data};
use rocket_contrib::json::JsonValue;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use validator::{Validate, ValidationError, ValidationErrors, ValidationErrorsKind};

// Submissions
// Simple Validation Macro generates following code automatically!
// ===============================================================
// impl SimpleValidation for RouteNewSubmission {}
//
// impl FromDataSimple for RouteNewSubmission {
//     type Error = ValidationErrors;
//
//     fn from_data(request: &Request, data: Data) -> Outcome<Self, Self::Error> {
//         SimpleValidation::from_data(request, data)
//     }
// }
#[derive(FromForm, Deserialize, Validate, SimpleValidation)]
pub struct RouteNewSubmission {
    #[validate(length(min = 1))]
    pub title: String,
    pub comment: String,
    #[serde(default)]
    pub attachments: NumberVec,
}

#[derive(Serialize, Deserialize, Validate, SimpleValidation)]
pub struct RouteUpdateReview {
    pub feedback: String,
    #[serde(default)]
    #[validate]
    pub points: Vec<RouteUpdatePoints>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct RouteUpdatePoints {
    pub id: u64,
    #[validate(range(min = 0.0, max = 100.0))]
    pub points: f64,
}

// Student & Teacher
#[derive(Serialize)]
pub struct RouteWorkshopResponse {
    pub(crate) id: u64,
    pub(crate) title: String,
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
            .0
            .into_iter()
            .map(|item| NewCriterion::from(item))
            .collect()
    }
}

#[derive(Deserialize)]
pub struct RouteCriterionVec(pub(crate) Vec<RouteCriterion>);

impl Validate for RouteCriterionVec {
    fn validate(&self) -> Result<(), ValidationErrors> {
        for criterion in &self.0 {
            match criterion.validate() {
                Ok(_) => {}
                Err(val_errors) => {
                    return Err(val_errors);
                }
            }
        }
        Ok(())
    }
}

// Expected format is ISO 8601 combined date and time without timezone like `2007-04-05T14:30:30`
// In JS that would mean creating a Date like this `(new Date()).toISOString().split(".")[0]`
// https://github.com/serde-rs/json/issues/531#issuecomment-479738561
#[derive(Deserialize, Serialize)]
pub struct Date(pub(crate) chrono::NaiveDateTime);

impl Validate for Date {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let current_date = Local::now().naive_local();
        if current_date > self.0 {
            let mut val_errors = ValidationErrors::new();
            let code = Cow::from("Deadline cannot be in the past".to_string());
            let mut params: HashMap<Cow<'static, str>, Value> = HashMap::new();
            params.insert(Cow::from("value"), serde_json::to_value(self.0).unwrap());
            val_errors.add(
                "",
                ValidationError {
                    code,
                    message: None,
                    params,
                },
            );
            return Err(val_errors);
        }
        Ok(())
    }
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

#[derive(FromForm, Deserialize, Validate, SimpleValidation)]
pub struct RouteNewWorkshop {
    #[validate(length(min = 1))]
    pub(crate) title: String,
    pub(crate) content: String,
    // See: https://serde.rs/string-or-struct.html
    #[validate]
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

#[derive(FromForm, Deserialize, Validate, SimpleValidation)]
pub struct RouteUpdateWorkshop {
    #[validate(length(min = 1))]
    pub(crate) title: String,
    pub(crate) content: String,
    #[validate]
    pub(crate) end: Date,
    pub(crate) teachers: NumberVec,
    pub(crate) students: NumberVec,
    #[validate]
    pub(crate) criteria: RouteCriterionVec,
    #[serde(default)]
    pub(crate) attachments: NumberVec,
}

// Users
#[derive(FromForm, Deserialize, Validate, SimpleValidation)]
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

#[derive(FromForm, Deserialize, Validate, SimpleValidation)]
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

#[derive(FromForm, Deserialize, Validate)]
#[validate(schema(function = "validate_route_search_student"))]
pub struct RouteSearchStudent {
    pub(crate) all: bool,
    pub(crate) id: Option<u64>,
    pub(crate) firstname: Option<String>,
    pub(crate) lastname: Option<String>,
    pub(crate) group: Option<String>,
}

// Struct Level validation
// See: https://github.com/Keats/validator/blob/master/validator_derive_tests/tests/schema.rs
fn validate_route_search_student(rss: &RouteSearchStudent) -> Result<(), ValidationError> {
    if rss.all {
        Ok(())
    } else if let Some(_) = &rss.id {
        Ok(())
    } else if let (Some(firstname), Some(lastname)) = (&rss.firstname, &rss.lastname) {
        if firstname.len() == 0 {
            return Err(ValidationError::new("Firstname cannot be empty"));
        }
        if lastname.len() == 0 {
            return Err(ValidationError::new("Lastname cannot be empty"));
        }
        Ok(())
    } else if let Some(_) = &rss.firstname {
        Err(ValidationError::new("Lastname not given"))
    } else if let Some(_) = &rss.lastname {
        Err(ValidationError::new("Firstname not given"))
    } else if let Some(group) = &rss.group {
        if group.len() == 0 {
            return Err(ValidationError::new("Group cannot be empty"));
        }
        Ok(())
    } else {
        Err(ValidationError::new("No parameters given"))
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

    pub fn conflict_with_error(error: DbError) -> Self {
        let json = json!({
            "ok": false,
            "error": error.description()
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

    pub fn forbidden_with_error(error: DbError) -> Self {
        let json = json!({
            "ok": false,
            "error": error.description()
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
        let validation_errors = validation_errs_to_str_vec(&validation_errors, None);
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
fn validation_errs_to_str_vec(ve: &ValidationErrors, root_name: Option<String>) -> Vec<String> {
    let root_name = root_name.unwrap_or(String::from(""));
    let mut error_msg: Vec<String> = Vec::new();
    for (name, error) in ve.errors() {
        if let ValidationErrorsKind::Struct(errors) = error {
            let root_name = format!("{}{}: ", root_name, name);
            let mut struct_errors = validation_errs_to_str_vec(errors, Some(root_name));
            error_msg.append(&mut struct_errors);
        } else if let ValidationErrorsKind::Field(errors) = error {
            // Needed for structs without named fields (e.g. Date)
            let name = if name.len() == 0 {
                String::from("")
            } else {
                String::from(format!("{}: ", name))
            };
            error_msg.push(format!(
                "{}{}errors: {}",
                root_name,
                name,
                errors
                    .iter()
                    .map(|ve| format!("{}: {:?}", ve.code, ve.params))
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
        }
    }
    error_msg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_new_submission_valid_data_ok() {
        let rts = RouteNewSubmission {
            title: "Great Title".to_string(),
            comment: "".to_string(),
            attachments: Default::default(),
        };
        assert!(rts.validate().is_ok());
    }

    #[test]
    fn route_new_submission_invalid_title_not_ok() {
        let rts = RouteNewSubmission {
            title: "".to_string(), // Empty title!
            comment: "".to_string(),
            attachments: Default::default(),
        };
        assert!(rts.validate().is_err());
    }

    #[test]
    fn route_update_points_valid_data_ok() {
        let rts = RouteUpdatePoints { id: 0, points: 0.0 };
        assert!(rts.validate().is_ok());
    }

    #[test]
    fn route_update_points_invalid_points_not_ok() {
        let rts = RouteUpdatePoints {
            id: 0,
            points: -0.1, // Under 0.0
        };
        let rts2 = RouteUpdatePoints {
            id: 0,
            points: 100.1, // Above 100.0
        };
        assert!(rts.validate().is_err());
        assert!(rts2.validate().is_err());
    }

    #[test]
    fn route_update_reviews_valid_data_ok() {
        let rur = RouteUpdateReview {
            feedback: "".to_string(),
            points: vec![],
        };
        assert!(rur.validate().is_ok());
    }

    #[test]
    fn route_update_reviews_invalid_rts_not_ok() {
        let rts = RouteUpdatePoints {
            id: 0,
            points: -0.1, // Under 0.0
        };
        let rur = RouteUpdateReview {
            feedback: "".to_string(),
            points: vec![rts],
        };
        assert!(rur.validate().is_err());
    }

    #[test]
    fn route_criterion_valid_data_ok() {
        let rc = RouteCriterion {
            title: "Great Title".to_string(),
            content: "".to_string(),
            weight: 0.0,
            kind: Kind::Point,
        };
        assert!(rc.validate().is_ok());
    }

    #[test]
    fn route_criterion_invalid_title_and_weights_not_ok() {
        let rc = RouteCriterion {
            title: "".to_string(),
            content: "".to_string(),
            weight: 0.0,
            kind: Kind::Point,
        };
        let rc2 = RouteCriterion {
            title: "Great Title".to_string(),
            content: "".to_string(),
            weight: -0.1,
            kind: Kind::Point,
        };
        let rc3 = RouteCriterion {
            title: "Great Title".to_string(),
            content: "".to_string(),
            weight: 100.1,
            kind: Kind::Point,
        };
        assert!(rc.validate().is_err());
        assert!(rc2.validate().is_err());
        assert!(rc3.validate().is_err());
    }

    #[test]
    fn route_criterion_vec_valid_data_ok() {
        let rcv = RouteCriterionVec { 0: vec![] };
        let rc = RouteCriterion {
            title: "Great Title".to_string(),
            content: "".to_string(),
            weight: 0.0,
            kind: Kind::Point,
        };
        let rcv2 = RouteCriterionVec { 0: vec![rc] };
        assert!(rcv.validate().is_ok());
        assert!(rcv2.validate().is_ok());
    }

    #[test]
    fn route_criterion_vec_invali_rc_not_ok() {
        let rc = RouteCriterion {
            title: "".to_string(),
            content: "".to_string(),
            weight: -0.1,
            kind: Kind::Point,
        };
        let rcv = RouteCriterionVec { 0: vec![rc] };
        assert!(rcv.validate().is_err());
    }

    #[test]
    fn date_valid_data_ok() {
        let future_date = Local::now().naive_local() + chrono::Duration::days(1);
        let d = Date { 0: future_date };
        assert!(d.validate().is_ok());
    }

    #[test]
    fn date_invalid_past_date_not_ok() {
        let past_date = Local::now().naive_local() - chrono::Duration::days(1);
        let d = Date { 0: past_date };
        assert!(d.validate().is_err());
    }

    #[test]
    fn route_new_workshop_valid_data_ok() {
        let future_date = Local::now().naive_local() + chrono::Duration::days(1);
        let d = Date { 0: future_date };
        let rc = RouteCriterion {
            title: "Great Title".to_string(),
            content: "".to_string(),
            weight: 0.0,
            kind: Kind::Point,
        };
        let rcv = RouteCriterionVec { 0: vec![rc] };
        let rnw = RouteNewWorkshop {
            title: "Great Title".to_string(),
            content: "".to_string(),
            end: d,
            anonymous: false,
            teachers: Default::default(),
            students: Default::default(),
            criteria: rcv,
            attachments: Default::default(),
        };
        assert!(rnw.validate().is_ok());
    }

    #[test]
    fn route_new_workshop_invalid_title_and_end_and_criteria_not_ok() {
        let future_date = Local::now().naive_local() + chrono::Duration::days(1);
        let d = Date { 0: future_date };
        let rc = RouteCriterion {
            title: "Great Title".to_string(),
            content: "".to_string(),
            weight: 0.0,
            kind: Kind::Point,
        };
        let rcv = RouteCriterionVec { 0: vec![rc] };
        let rnw = RouteNewWorkshop {
            title: "".to_string(),
            content: "".to_string(),
            end: d,
            anonymous: false,
            teachers: Default::default(),
            students: Default::default(),
            criteria: rcv,
            attachments: Default::default(),
        };
        let past_date = Local::now().naive_local() - chrono::Duration::days(1);
        let d = Date { 0: past_date };
        let rc = RouteCriterion {
            title: "Great Title".to_string(),
            content: "".to_string(),
            weight: 0.0,
            kind: Kind::Point,
        };
        let rcv = RouteCriterionVec { 0: vec![rc] };
        let rnw2 = RouteNewWorkshop {
            title: "Great Title".to_string(),
            content: "".to_string(),
            end: d,
            anonymous: false,
            teachers: Default::default(),
            students: Default::default(),
            criteria: rcv,
            attachments: Default::default(),
        };
        let future_date = Local::now().naive_local() + chrono::Duration::days(1);
        let d = Date { 0: future_date };
        let rc = RouteCriterion {
            title: "".to_string(),
            content: "".to_string(),
            weight: -0.1,
            kind: Kind::Point,
        };
        let rcv = RouteCriterionVec { 0: vec![rc] };
        let rnw3 = RouteNewWorkshop {
            title: "Great Title".to_string(),
            content: "".to_string(),
            end: d,
            anonymous: false,
            teachers: Default::default(),
            students: Default::default(),
            criteria: rcv,
            attachments: Default::default(),
        };
        assert!(rnw.validate().is_err());
        assert!(rnw2.validate().is_err());
        assert!(rnw3.validate().is_err());
    }

    #[test]
    fn route_update_workshop_valid_data_ok() {
        let future_date = Local::now().naive_local() + chrono::Duration::days(1);
        let d = Date { 0: future_date };
        let rc = RouteCriterion {
            title: "Great Title".to_string(),
            content: "".to_string(),
            weight: 0.0,
            kind: Kind::Point,
        };
        let rcv = RouteCriterionVec { 0: vec![rc] };
        let rup = RouteUpdateWorkshop {
            title: "Great Title".to_string(),
            content: "".to_string(),
            end: d,
            teachers: Default::default(),
            students: Default::default(),
            criteria: rcv,
            attachments: Default::default(),
        };
        assert!(rup.validate().is_ok());
    }

    #[test]
    fn route_update_workshop_invalid_title_and_end_and_criteria_not_ok() {
        let future_date = Local::now().naive_local() + chrono::Duration::days(1);
        let d = Date { 0: future_date };
        let rc = RouteCriterion {
            title: "Great Title".to_string(),
            content: "".to_string(),
            weight: 0.0,
            kind: Kind::Point,
        };
        let rcv = RouteCriterionVec { 0: vec![rc] };
        let rup = RouteUpdateWorkshop {
            title: "".to_string(),
            content: "".to_string(),
            end: d,
            teachers: Default::default(),
            students: Default::default(),
            criteria: rcv,
            attachments: Default::default(),
        };
        let past_date = Local::now().naive_local() - chrono::Duration::days(1);
        let d = Date { 0: past_date };
        let rc = RouteCriterion {
            title: "Great Title".to_string(),
            content: "".to_string(),
            weight: 0.0,
            kind: Kind::Point,
        };
        let rcv = RouteCriterionVec { 0: vec![rc] };
        let rup2 = RouteUpdateWorkshop {
            title: "Great Title".to_string(),
            content: "".to_string(),
            end: d,
            teachers: Default::default(),
            students: Default::default(),
            criteria: rcv,
            attachments: Default::default(),
        };
        let future_date = Local::now().naive_local() + chrono::Duration::days(1);
        let d = Date { 0: future_date };
        let rc = RouteCriterion {
            title: "".to_string(),
            content: "".to_string(),
            weight: -0.1,
            kind: Kind::Point,
        };
        let rcv = RouteCriterionVec { 0: vec![rc] };
        let rup3 = RouteUpdateWorkshop {
            title: "Great Title".to_string(),
            content: "".to_string(),
            end: d,
            teachers: Default::default(),
            students: Default::default(),
            criteria: rcv,
            attachments: Default::default(),
        };
        assert!(rup.validate().is_err());
        assert!(rup2.validate().is_err());
        assert!(rup3.validate().is_err());
    }

    #[test]
    fn route_create_student_valid_data_ok() {
        let rcs = RouteCreateStudent {
            username: "User".to_string(),
            firstname: "Max".to_string(),
            lastname: "Mustermann".to_string(),
            password: "1234".to_string(),
            unit: "5A".to_string(),
        };
        assert!(rcs.validate().is_ok());
    }

    #[test]
    fn route_create_student_invalid_username_and_firstname_and_lastname_and_password_not_ok() {
        let rcs = RouteCreateStudent {
            username: "".to_string(),
            firstname: "Max".to_string(),
            lastname: "Mustermann".to_string(),
            password: "1234".to_string(),
            unit: "5A".to_string(),
        };
        let rcs2 = RouteCreateStudent {
            username: "User".to_string(),
            firstname: "".to_string(),
            lastname: "Mustermann".to_string(),
            password: "1234".to_string(),
            unit: "5A".to_string(),
        };
        let rcs3 = RouteCreateStudent {
            username: "User".to_string(),
            firstname: "Max".to_string(),
            lastname: "".to_string(),
            password: "1234".to_string(),
            unit: "5A".to_string(),
        };
        let rcs4 = RouteCreateStudent {
            username: "User".to_string(),
            firstname: "Max".to_string(),
            lastname: "Mustermann".to_string(),
            password: "".to_string(),
            unit: "5A".to_string(),
        };
        assert!(rcs.validate().is_err());
        assert!(rcs2.validate().is_err());
        assert!(rcs3.validate().is_err());
        assert!(rcs4.validate().is_err());
    }

    #[test]
    fn route_create_teacher_valid_data_ok() {
        let rct = RouteCreateTeacher {
            username: "User".to_string(),
            firstname: "Max".to_string(),
            lastname: "Mustermann".to_string(),
            password: "1234".to_string(),
        };
        assert!(rct.validate().is_ok());
    }

    #[test]
    fn route_create_teacher_invalid_username_and_firstname_and_lastname_and_password_not_ok() {
        let rct = RouteCreateTeacher {
            username: "".to_string(),
            firstname: "Max".to_string(),
            lastname: "Mustermann".to_string(),
            password: "1234".to_string(),
        };
        let rct2 = RouteCreateTeacher {
            username: "User".to_string(),
            firstname: "".to_string(),
            lastname: "Mustermann".to_string(),
            password: "1234".to_string(),
        };
        let rct3 = RouteCreateTeacher {
            username: "User".to_string(),
            firstname: "Max".to_string(),
            lastname: "".to_string(),
            password: "1234".to_string(),
        };
        let rct4 = RouteCreateTeacher {
            username: "User".to_string(),
            firstname: "Max".to_string(),
            lastname: "Mustermann".to_string(),
            password: "".to_string(),
        };
        assert!(rct.validate().is_err());
        assert!(rct2.validate().is_err());
        assert!(rct3.validate().is_err());
        assert!(rct4.validate().is_err());
    }

    #[test]
    fn route_search_student_valid_all_search_ok() {
        let rss = RouteSearchStudent {
            all: true,
            id: None,
            firstname: None,
            lastname: None,
            group: None,
        };
        assert!(rss.validate().is_ok());
    }

    #[test]
    fn route_search_student_valid_id_search_ok() {
        let rss = RouteSearchStudent {
            all: false,
            id: Some(1),
            firstname: None,
            lastname: None,
            group: None,
        };
        assert!(rss.validate().is_ok());
    }

    #[test]
    fn route_search_student_valid_student_search_ok() {
        let rss = RouteSearchStudent {
            all: false,
            id: None,
            firstname: Some("Max".to_string()),
            lastname: Some("Mustermann".to_string()),
            group: None,
        };
        assert!(rss.validate().is_ok());
    }

    #[test]
    fn route_search_student_valid_group_search_ok() {
        let rss = RouteSearchStudent {
            all: false,
            id: None,
            firstname: None,
            lastname: None,
            group: Some("5A".to_string()),
        };
        assert!(rss.validate().is_ok());
    }

    #[test]
    fn route_search_student_no_parameters_not_ok() {
        let rss = RouteSearchStudent {
            all: false,
            id: None,
            firstname: None,
            lastname: None,
            group: None,
        };
        assert!(rss.validate().is_err());
    }

    #[test]
    fn route_search_student_no_firstname_not_ok() {
        let rss = RouteSearchStudent {
            all: false,
            id: None,
            firstname: None,
            lastname: Some("Mustermann".to_string()),
            group: None,
        };
        assert!(rss.validate().is_err());
    }

    #[test]
    fn route_search_student_no_lastname_not_ok() {
        let rss = RouteSearchStudent {
            all: false,
            id: None,
            firstname: Some("Max".to_string()),
            lastname: None,
            group: None,
        };
        assert!(rss.validate().is_err());
    }

    #[test]
    fn route_search_student_invalid_firstname_not_ok() {
        let rss = RouteSearchStudent {
            all: false,
            id: None,
            firstname: Some("".to_string()),
            lastname: Some("Mustermann".to_string()),
            group: None,
        };
        assert!(rss.validate().is_err());
    }

    #[test]
    fn route_search_student_invalid_lastname_not_ok() {
        let rss = RouteSearchStudent {
            all: false,
            id: None,
            firstname: Some("Max".to_string()),
            lastname: Some("".to_string()),
            group: None,
        };
        assert!(rss.validate().is_err());
    }

    #[test]
    fn route_search_student_invalid_firstname_and_lastname_not_ok() {
        let rss = RouteSearchStudent {
            all: false,
            id: None,
            firstname: Some("".to_string()),
            lastname: Some("".to_string()),
            group: None,
        };
        assert!(rss.validate().is_err());
    }

    #[test]
    fn route_search_student_invalid_group_not_ok() {
        let rss = RouteSearchStudent {
            all: false,
            id: None,
            firstname: None,
            lastname: None,
            group: Some("".to_string()),
        };
        assert!(rss.validate().is_err());
    }
}
