use crate::{db, IprpDB};
use rocket::http::Status;
use rocket_contrib::json;

#[derive(FromForm, Deserialize)]
pub struct SearchStudent {
    firstname: String,
    lastname: String,
}

#[post("/teachers/search/student", format = "json", data = "<create_info>")]
pub fn create_user(
    conn: IprpDB,
    create_info: json::Json<SearchStudent>,
) -> Result<json::Json<u64>, Status> {
    // TODO db search
    /*
    let hashed_password = crate::auth::crypto::hash_password(&create_info.0.password);
    let user = db::users::create_user(
        &*conn,
        create_info.0.username,
        create_info.0.firstname,
        create_info.0.lastname,
        hashed_password,
    );
    return match user {
        Ok(user) => Ok(json::Json(user.id)),
        Err(_) => Err(Status::Conflict),
    };*/
    Err(Status::NotFound)
}
