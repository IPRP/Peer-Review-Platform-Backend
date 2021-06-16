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

#[get("/student/workshops")]
pub fn workshops(user: User, conn: IprpDB) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Teacher {
        return Err(ApiResponse::forbidden());
    }

    let workshops = db::workshops::get_by_user(&*conn, user.id);
    let workshop_infos = workshops
        .into_iter()
        .map(|ws| WorkshopResponse {
            id: ws.id,
            title: ws.title,
        })
        .collect::<Vec<WorkshopResponse>>();
    Ok(Json(json!({
        "ok": true,
        "workshops": workshop_infos
    })))
}

#[get("/student/todos")]
pub fn todos(user: User, conn: IprpDB) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Teacher {
        return Err(ApiResponse::forbidden());
    }

    let todos = db::todos::get(&*conn, user.id);
    if todos.is_err() {
        return Err(ApiResponse::bad_request());
    }
    let todos = todos.unwrap();

    Ok(Json(json!({
        "ok": true,
        "reviews": todos.reviews,
        "submissions": todos.submissions
    })))
}

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
