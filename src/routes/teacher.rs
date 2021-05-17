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
        let firstname = &*search_info.firstname.as_ref().unwrap();
        let lastname = &*search_info.lastname.as_ref().unwrap();

        let user = db::users::get_student_by_firstname_lastname(&*conn, firstname, lastname);

        match user {
            Ok(user) => Ok(content::Json(format!(
                r#"
{{ 
    'ok': true,
    'id': {} 
}}
                "#,
                user.id
            ))),
            Err(_) => Err(Status::NotFound),
        }
    } else if search_info.group.is_some() {
        let unit = &*search_info.group.as_ref().unwrap();
        let users = db::users::get_students_by_unit(&*conn, unit);

        match users {
            Ok(users) => {
                let mut ids: Vec<u64> = Vec::new();
                for user in users {
                    ids.push(user.id);
                }
                Ok(content::Json(format!(
                    r#"
{{
    'ok': true,
    'id': {:?}
}}
                "#,
                    ids
                )))
            }
            Err(_) => Err(Status::NotFound),
        }
    } else {
        Err(Status::BadRequest)
    }
}
