use crate::models::{Kind, NewCriterion, Role, User};
use crate::routes::models::{ApiResponse, NumberVec, WorkshopResponse};
use crate::{db, IprpDB};
use chrono::{Local, Utc};
use diesel::result::Error;
use rocket::http::{RawStr, Status};
use rocket::request::FromFormValue;
use rocket::response::content;
use rocket_contrib::json::{Json, JsonValue};
use serde::{de, Deserialize, Deserializer};
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

#[post(
    "/submission/<workshop_id>",
    format = "json",
    data = "<new_submission>"
)]
pub fn create_submission(
    user: User,
    conn: IprpDB,
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

#[get("/submission/<submission_id>")]
pub fn get_submission(
    user: User,
    conn: IprpDB,
    submission_id: u64,
) -> Result<Json<JsonValue>, ApiResponse> {
    if db::submissions::is_owner(&*conn, submission_id, user.id) {
        let submission = db::submissions::get_own_submission(&*conn, submission_id, user.id);
        match submission {
            Ok(submission) => Ok(Json(json!({
                "ok": true,
                "title": submission.title,
                "comment": submission.comment,
                "attachments": submission.attachments,
                "locked": submission.locked,
                "reviewsDone": submission.locked,
                "points": submission.points,
                "maxPoints": submission.max_points,
                "lastname": submission.lastname,
                "firstname": submission.firstname
            }))),
            Err(_) => Err(ApiResponse::forbidden()),
        }
    } else {
        Err(ApiResponse::bad_request())
    }
}
