use crate::db::ReviewTimespan;
use crate::models::{Kind, NewCriterion, Role, User};
use crate::routes::models::{ApiResponse, NumberVec, WorkshopResponse};
use crate::utils;
use crate::{db, IprpDB};
use chrono::{Local, Utc};
use diesel::result::Error;
use rocket::http::{RawStr, Status};
use rocket::request::FromFormValue;
use rocket::response::content;
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;
use std::fmt::Display;
use std::fs::read;
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;

#[derive(FromForm, Deserialize)]
pub struct NewSubmission {
    title: String,
    comment: String,
    attachments: NumberVec,
}

/// Create new submission.
#[post(
    "/submission/<workshop_id>",
    format = "json",
    data = "<new_submission>"
)]
pub fn create_submission(
    user: User,
    conn: IprpDB,
    review_timespan: State<ReviewTimespan>,
    workshop_id: u64,
    new_submission: Json<NewSubmission>,
) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Teacher {
        return Err(ApiResponse::forbidden());
    }
    // Get current date
    // See: https://stackoverflow.com/a/48237707/12347616
    // And: https://stackoverflow.com/q/28747694/12347616
    let date = Local::now().naive_local();

    let submission = db::submissions::create(
        &*conn,
        review_timespan.inner(),
        new_submission.0.title,
        new_submission.0.comment,
        Vec::from(new_submission.0.attachments),
        date,
        user.id,
        workshop_id,
    );

    match submission {
        Ok(submission) => Ok(Json(json!({
            "ok": true,
            "id": submission.id
        }))),
        Err(_) => Err(ApiResponse::conflict()),
    }
}

/// Get existing submission.
#[get("/submission/<submission_id>")]
pub fn get_submission(
    user: User,
    conn: IprpDB,
    submission_id: u64,
) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Teacher {
        let submission = db::submissions::get_teacher_submission(&*conn, submission_id);
        match submission {
            Ok(submission) => {
                let mut json_response = serde_json::to_value(submission).unwrap();
                let json_additional_info = json!({
                    "ok": true
                });
                utils::json::merge(&mut json_response, &*json_additional_info);
                Ok(Json(JsonValue::from(json_response)))
            }
            Err(_) => Err(ApiResponse::forbidden()),
        }
    } else if db::submissions::is_owner(&*conn, submission_id, user.id) {
        let submission = db::submissions::get_own_submission(&*conn, submission_id);
        match submission {
            Ok(submission) => {
                let mut json_response = serde_json::to_value(submission).unwrap();
                let json_additional_info = json!({
                    "ok": true
                });
                utils::json::merge(&mut json_response, &*json_additional_info);
                Ok(Json(JsonValue::from(json_response)))
            }
            Err(_) => Err(ApiResponse::forbidden()),
        }
    } else {
        if db::reviews::is_reviewer(&*conn, submission_id, user.id) {
            let submission =
                db::submissions::get_student_submission(&*conn, submission_id, user.id);
            match submission {
                Ok(submission) => {
                    let mut json_response = serde_json::to_value(submission).unwrap();
                    let json_additional_info = json!({
                        "ok": true
                    });
                    utils::json::merge(&mut json_response, &*json_additional_info);
                    Ok(Json(JsonValue::from(json_response)))
                }
                Err(_) => Err(ApiResponse::forbidden()),
            }
        } else {
            Err(ApiResponse::bad_request())
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateReview {
    pub feedback: String,
    pub points: Vec<UpdatePoints>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdatePoints {
    pub id: u64,
    pub points: f64,
}

/// Update existing review.
#[put("/review/<review_id>", format = "json", data = "<update_review>")]
pub fn update_review(
    user: User,
    conn: IprpDB,
    review_id: u64,
    update_review: Json<UpdateReview>,
) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Teacher {
        return Err(ApiResponse::forbidden());
    }

    let res = db::reviews::update(&*conn, update_review.0, review_id, user.id);

    match res {
        true => Ok(Json(json!({
            "ok": true
        }))),
        false => Err(ApiResponse::forbidden()),
    }
}

/// Get specific review.
#[get("/review/<review_id>")]
pub fn get_review(
    user: User,
    conn: IprpDB,
    review_id: u64,
) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Teacher || db::reviews::is_owner(&*conn, review_id, user.id) {
        // If user is teacher or reviewer owner return review with name
        let review = db::reviews::get_full_review_with_names(&*conn, review_id);
        match review {
            Ok(review) => {
                let mut json_response = serde_json::to_value(review).unwrap();
                let json_additional_info = json!({
                    "ok": true
                });
                utils::json::merge(&mut json_response, &*json_additional_info);
                Ok(Json(JsonValue::from(json_response)))
            }
            Err(_) => Err(ApiResponse::forbidden()),
        }
    } else if db::reviews::is_submission_owner(&*conn, review_id, user.id) {
        // If user is only the submission owner, check first if the workshop is anonymous or not
        let review = db::reviews::get_full_review(&*conn, review_id);
        match review {
            Ok(review) => {
                let mut json_response = serde_json::to_value(review).unwrap();
                let json_additional_info = json!({
                    "ok": true
                });
                utils::json::merge(&mut json_response, &*json_additional_info);
                Ok(Json(JsonValue::from(json_response)))
            }
            Err(_) => Err(ApiResponse::forbidden()),
        }
    } else {
        Err(ApiResponse::bad_request())
    }
}
