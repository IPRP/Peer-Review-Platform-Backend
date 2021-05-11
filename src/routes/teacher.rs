use crate::models::User;
use crate::{db, IprpDB};
use rocket::http::Status;
use rocket_contrib::json;

#[derive(FromForm, Deserialize)]
pub struct SearchStudent {
    firstname: String,
    lastname: String,
}

#[get("/teachers/search/student", format = "json", data = "<search_info>")]
pub fn search_student(
    _user: User,
    conn: IprpDB,
    search_info: json::Json<SearchStudent>,
) -> Result<json::Json<u64>, Status> {
    // TODO role check

    // TODO db search
    let user = db::users::get_by_firstname_lastname(
        &*conn,
        &*search_info.firstname,
        &*search_info.lastname,
    );

    match user {
        Ok(user) => Ok(json::Json(user.id)),
        Err(_) => Err(Status::NotFound),
    }
}
