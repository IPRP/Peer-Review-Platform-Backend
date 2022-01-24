use crate::db::models::*;
use crate::routes::models::{ApiResponse, RouteNewSubmission, RouteUpdateReview};
use crate::utils;
use crate::{db, IprpDB};
use chrono::Local;

use crate::routes::error::{RouteError, RouteErrorKind};
use crate::utils::error::AppError;
use rocket_contrib::json::{Json, JsonValue};

/// Create new submission.
#[post(
    "/submission/<workshop_id>",
    format = "json",
    data = "<new_submission>"
)]
pub fn create_submission(
    user: User,
    conn: IprpDB,
    workshop_id: u64,
    new_submission: RouteNewSubmission,
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
        new_submission.title,
        new_submission.comment,
        Vec::from(new_submission.attachments),
        date,
        user.id,
        workshop_id,
    );

    match submission {
        Ok(submission) => Ok(Json(json!({
            "ok": true,
            "id": submission.id
        }))),
        Err(err) => {
            //println!("Error occurred {}", err);
            err.print_stacktrace();
            Err(ApiResponse::conflict_with_error(err))
        }
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
            Err(err) => {
                println!("Error occurred {}", err);
                Err(ApiResponse::forbidden_with_error(err))
            }
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
            Err(err) => {
                println!("Error occurred {}", err);
                Err(ApiResponse::forbidden_with_error(err))
            }
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
                Err(err) => {
                    println!("Error occurred {}", err);
                    Err(ApiResponse::forbidden_with_error(err))
                }
            }
        } else {
            let err = RouteError::new(
                RouteErrorKind::BadRequest,
                format!(
                    "User {} is not a teacher, owner or reviewer for Submission {}",
                    user.id, submission_id
                ),
            );
            println!("Error occurred {}", err);
            Err(ApiResponse::bad_request_with_error(err))
        }
    }
}

/// Update existing submission.
#[put(
    "/submission/<submission_id>",
    format = "json",
    data = "<new_submission>"
)]
pub fn update_submission(
    user: User,
    conn: IprpDB,
    submission_id: u64,
    new_submission: RouteNewSubmission,
) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Teacher {
        return Err(ApiResponse::forbidden());
    }

    let update = db::submissions::update(
        &*conn,
        submission_id,
        user.id,
        new_submission.title,
        new_submission.comment,
        Vec::from(new_submission.attachments),
    );

    match update {
        Ok(_) => Ok(Json(json!({
            "ok": true,
        }))),
        Err(err) => {
            err.print_stacktrace();
            Err(ApiResponse::bad_request_with_error(err))
        }
    }
}

/// Update existing review.
#[put("/review/<review_id>", format = "json", data = "<update_review>")]
pub fn update_review(
    user: User,
    conn: IprpDB,
    review_id: u64,
    update_review: RouteUpdateReview,
) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Teacher {
        return Err(ApiResponse::forbidden());
    }

    let res = db::reviews::update(&*conn, update_review, review_id, user.id);

    match res {
        Ok(_) => Ok(Json(json!({
            "ok": true
        }))),
        Err(err) => {
            println!("Error occurred {}", err);
            Err(ApiResponse::forbidden_with_error(err))
        }
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
