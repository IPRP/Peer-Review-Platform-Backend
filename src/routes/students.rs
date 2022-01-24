use crate::db::models::*;
use crate::routes::models::{ApiResponse, RouteWorkshopResponse};
use crate::{db, IprpDB};

use crate::utils::error::AppError;
use rocket_contrib::json::{Json, JsonValue};

/// Get all workshops.
#[get("/student/workshops")]
pub fn workshops(user: User, conn: IprpDB) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Teacher {
        return Err(ApiResponse::forbidden());
    }

    let workshops = db::workshops::get_by_user(&*conn, user.id);
    let workshop_infos = workshops
        .into_iter()
        .map(|ws| RouteWorkshopResponse {
            id: ws.id,
            title: ws.title,
        })
        .collect::<Vec<RouteWorkshopResponse>>();
    Ok(Json(json!({
        "ok": true,
        "workshops": workshop_infos
    })))
}

/// Get specific workshop.
#[get("/student/workshop/<workshop_id>")]
pub fn workshop(
    user: User,
    conn: IprpDB,
    workshop_id: u64,
) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Teacher {
        return Err(ApiResponse::forbidden());
    }

    let workshop = db::workshops::get_student_workshop(&*conn, workshop_id, user.id);
    match workshop {
        Ok(workshop) => Ok(Json(json!({
            "ok": true,
            "workshop": workshop
        }))),
        Err(err) => {
            err.print_stacktrace();
            Err(ApiResponse::not_found_with_error(err))
        }
    }
}

/// Get student TODOs.
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
