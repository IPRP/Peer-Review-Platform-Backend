use crate::models::{Kind, NewCriterion, Role, User};
use crate::routes::models::{ApiResponse, WorkshopResponse};
use crate::{db, IprpDB};
use chrono::Utc;
use diesel::result::Error;
use rocket::http::{RawStr, Status};
use rocket::request::FromFormValue;
use rocket::response::content;
use rocket_contrib::json::{Json, JsonValue};
use serde::{de, Deserialize, Deserializer};
use std::fmt::Display;
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;

#[get("/student/workshops")]
pub fn workshops(user: User, conn: IprpDB) -> Result<Json<JsonValue>, ApiResponse> {
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
