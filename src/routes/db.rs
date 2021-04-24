use crate::models::User;
use crate::{db, IprpDB};
use rocket::http::Status;
use rocket::response::content;
use rocket_contrib::json;
use serde::{Deserialize, Serialize};

/*
#[get("/add")]
pub fn create_post(conn: TestDB) -> content::Json<String> {
    // Create a post
    let title = String::from("Something");
    let body = String::from("Great");
    // Save it to db
    let post = db::posts::create_post(&*conn, &title, &body);
    // Send JSON
    // See: https://rocket.rs/v0.4/guide/responses/
    content::Json(format!("{{ 'id': {} }}", &post.id))
}

#[get("/event")]
pub fn create_event(conn: TestDB) -> String {
    db::posts::create_post_event(&*conn);
    String::from("ok!")
}*/

#[derive(FromForm, Deserialize)]
pub struct CreateInfo {
    username: String,
    password: String,
}

#[post("/users/create", format = "json", data = "<create_info>")]
pub fn create_user(
    conn: IprpDB,
    create_info: json::Json<CreateInfo>,
) -> Result<json::Json<u64>, Status> {
    let hashed_password = crate::auth::crypto::hash_password(&create_info.0.password);
    let user = db::users::create_user(&*conn, create_info.0.username, hashed_password);
    return match user {
        Ok(user) => Ok(json::Json(user.id)),
        Err(_) => Err(Status::Conflict),
    };
}
