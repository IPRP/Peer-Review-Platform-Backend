use crate::models::User;
use crate::{db, IprpDB};
use rocket::http::{Cookie, Cookies, Status};

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

#[derive(FromForm, Deserialize)]
pub struct CreateStudent {
    username: String,
    firstname: String,
    lastname: String,
    password: String,
    #[serde(rename(deserialize = "group"))]
    unit: String,
}

/// Create new student account.
/// Only accessible with "admin" account.
#[post("/users/student", format = "json", data = "<create_info>")]
pub fn create_student(
    user: User,
    conn: IprpDB,
    create_info: json::Json<CreateStudent>,
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

#[derive(FromForm, Deserialize)]
pub struct CreateTeacher {
    username: String,
    firstname: String,
    lastname: String,
    password: String,
}

/// Create new teacher account.
/// Only accessible with "admin" account.
#[post("/users/teacher", format = "json", data = "<create_info>")]
pub fn create_teacher(
    user: User,
    conn: IprpDB,
    create_info: json::Json<CreateTeacher>,
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
