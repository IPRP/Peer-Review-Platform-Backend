use crate::models::User;
use crate::{db, IprpDB};
use rocket::http::Status;
use rocket::response::content;
use rocket_contrib::json;

#[derive(FromForm, Deserialize)]
pub struct SearchStudent {
    firstname: Option<String>,
    lastname: Option<String>,
    group: Option<String>,
}

#[get("/teachers/search/student", format = "json", data = "<search_info>")]
pub fn search_student(
    _user: User,
    conn: IprpDB,
    search_info: json::Json<SearchStudent>,
) -> Result<content::Json<String>, Status> {
    // TODO role check

    if search_info.firstname.is_some() && search_info.lastname.is_some() {
        println!("{}", &*search_info.lastname.as_ref().unwrap());
        println!("{}", &*search_info.firstname.as_ref().unwrap());
        let user = db::users::get_student_by_firstname_lastname(
            &*conn,
            &*search_info.firstname.as_ref().unwrap(),
            &*search_info.lastname.as_ref().unwrap(),
        );

        match user {
            Ok(user) => Ok(content::Json(format!("{{ 'id': {} }}", user.id))),
            Err(_) => Err(Status::NotFound),
        }
    } else if search_info.group.is_some() {
        println!("{}", &*search_info.group.as_ref().unwrap());
        Ok(content::Json(format!("{{ 'id': {} }}", 10)))
    } else {
        Err(Status::BadRequest)
    }
}
