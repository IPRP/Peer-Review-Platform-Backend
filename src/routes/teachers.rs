use crate::db::models::*;
use crate::routes::models::{
    ApiResponse, Date, NumberVec, RouteCriterionVec, RouteNewWorkshop, RouteSearchStudent,
    RouteUpdateWorkshop, RouteWorkshopResponse,
};
use crate::{db, IprpDB};

use rocket::http::RawStr;
use rocket::request::FromFormValue;
use rocket::State;
use validator::Validate;

use crate::db::ReviewTimespan;
use crate::utils::error::AppError;
use rocket_contrib::json::{Json, JsonValue};

/// Get all workshops.
#[get("/teacher/workshops")]
pub fn workshops(user: User, conn: IprpDB) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Student {
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
#[get("/teacher/workshop/<workshop_id>")]
pub fn workshop(
    user: User,
    conn: IprpDB,
    workshop_id: u64,
) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Student {
        return Err(ApiResponse::forbidden());
    }

    let workshop = db::workshops::get_teacher_workshop(&*conn, workshop_id);
    match workshop {
        Ok(workshop) => Ok(Json(json!({
            "ok": true,
            "workshop": workshop
        }))),
        Err(_) => Err(ApiResponse::not_found()),
    }
}

/// Create new workshop.
#[post("/teacher/workshop", format = "json", data = "<new_workshop>")]
pub fn create_workshop(
    user: User,
    conn: IprpDB,
    mut new_workshop: RouteNewWorkshop,
    review_timespan: State<ReviewTimespan>,
) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Student {
        return Err(ApiResponse::forbidden());
    }

    // Add teacher, who wants to create the workshop, to the teachers list
    // if not already present
    if !new_workshop.teachers.0.contains(&user.id) {
        new_workshop.teachers.0.push(user.id);
    }

    // Check if proper timespan was already given
    // If not, use default value
    if new_workshop.review_timespan.is_none() {
        new_workshop.review_timespan = Some(review_timespan.inner().in_minutes())
    }

    let workshop = db::workshops::create(
        &*conn,
        user.id,
        new_workshop.title,
        new_workshop.content,
        new_workshop.end.0,
        new_workshop.review_timespan.unwrap(),
        new_workshop.anonymous,
        Vec::from(new_workshop.teachers),
        Vec::from(new_workshop.students),
        Vec::from(new_workshop.criteria),
        Vec::from(new_workshop.attachments),
    );
    match workshop {
        Ok(workshop) => Ok(Json(json!({
            "ok": true,
            "id": workshop.id
        }))),
        Err(err) => {
            err.print_stacktrace();
            Err(ApiResponse::conflict_with_error(err))
        }
    }
}

/// Update workshop.
#[put(
    "/teacher/workshop/<workshop_id>",
    format = "json",
    data = "<update_workshop>"
)]
pub fn update_workshop(
    user: User,
    conn: IprpDB,
    workshop_id: u64,
    mut update_workshop: RouteUpdateWorkshop,
    review_timespan: State<ReviewTimespan>,
) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Student {
        return Err(ApiResponse::forbidden());
    }

    // Add teacher, who wants to update the workshop, to the teachers list
    // if not already present
    if !update_workshop.teachers.0.contains(&user.id) {
        update_workshop.teachers.0.push(user.id);
    }

    // Check if proper timespan was already given
    // If not, use default value
    if update_workshop.review_timespan.is_none() {
        update_workshop.review_timespan = Some(review_timespan.inner().in_minutes())
    }

    let workshop = db::workshops::update(
        &*conn,
        user.id,
        workshop_id,
        update_workshop.title,
        update_workshop.content,
        update_workshop.end.0,
        update_workshop.review_timespan.unwrap(),
        Vec::from(update_workshop.teachers),
        Vec::from(update_workshop.students),
        Vec::from(update_workshop.criteria),
        Vec::from(update_workshop.attachments),
    );
    match workshop {
        Ok(_) => Ok(Json(json!({
            "ok": true,
        }))),
        Err(err) => {
            err.print_stacktrace();
            Err(ApiResponse::conflict_with_error(err))
        }
    }
}

/// Delete existing workshop.
#[delete("/teacher/workshop/<id>")]
pub fn delete_workshop(user: User, conn: IprpDB, id: u64) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Student {
        return Err(ApiResponse::forbidden());
    }

    let result = db::workshops::delete(&*conn, id);
    match result {
        Ok(_) => Ok(Json(json!({"ok": true}))),
        Err(_) => Err(ApiResponse::not_found()),
    }
}

/// Search students.
/// Different Query Parameter yield different results.
#[get("/teacher/search/student?<all>&<id>&<firstname>&<lastname>&<group>")]
pub fn search_student(
    user: User,
    conn: IprpDB,
    all: Option<bool>,
    id: Option<u64>,
    firstname: Option<String>,
    lastname: Option<String>,
    group: Option<String>,
) -> Result<Json<JsonValue>, ApiResponse> {
    let all = all.unwrap_or(false);
    let search_info = RouteSearchStudent {
        all,
        id,
        firstname,
        lastname,
        group,
    };

    if user.role == Role::Student {
        return Err(ApiResponse::forbidden());
    }

    // Struct Level Validation
    if let Err(val_errors) = search_info.validate() {
        return Err(ApiResponse::unprocessable_entity(&val_errors));
    }

    if search_info.all {
        let users = db::users::get_all_students(&*conn);
        match users {
            Ok(users) => Ok(Json(json!({
                "ok": true,
                "students": users
            }))),
            Err(_) => Err(ApiResponse::bad_request()),
        }
    } else if search_info.id.is_some() {
        let id = search_info.id.unwrap();
        let user = db::users::get_student_by_id(&*conn, id);

        match user {
            Ok(user) => Ok(Json(json!({
                "ok": true,
                "firstname": user.firstname,
                "lastname": user.lastname
            }))),
            Err(_) => Err(ApiResponse::bad_request()),
        }
    } else if search_info.firstname.is_some() && search_info.lastname.is_some() {
        let firstname = &*search_info.firstname.as_ref().unwrap();
        let lastname = &*search_info.lastname.as_ref().unwrap();
        let user = db::users::get_student_by_firstname_lastname(&*conn, firstname, lastname);

        match user {
            Ok(user) => Ok(Json(json!({
                "ok": true,
                "id": user.id
            }))),
            Err(_) => Err(ApiResponse::bad_request()),
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
                Ok(Json(json!({
                    "ok": true,
                    "ids": ids
                })))
            }
            Err(_) => Err(ApiResponse::bad_request()),
        }
    } else {
        Err(ApiResponse::bad_request())
    }
}

// See: https://api.rocket.rs/v0.4/rocket/request/trait.FromFormValue.html#example
impl<'v> FromFormValue<'v> for Date {
    type Error = &'v RawStr;

    fn from_form_value(_form_value: &'v RawStr) -> Result<Self, Self::Error> {
        unimplemented!()
    }

    /*
    fn from_form_value(form_value: &'v RawStr) -> Result<Date, &'v RawStr> {
        // See: https://docs.rs/chrono/0.4.19/chrono/naive/struct.NaiveDate.html#method.parse_from_str
        let date = chrono::NaiveDate::parse_from_str("2015-09-05", "%Y-%m-%d");
        match date {
            Ok(date) => Ok(Date(date)),
            _ => Err(RawStr::from_str(
                "Date should be formatted `%Y-%m-%d` like `2015-09-05`",
            )),
        }
    }*/
}

impl<'v> FromFormValue<'v> for NumberVec {
    type Error = &'v RawStr;

    fn from_form_value(_form_value: &'v RawStr) -> Result<Self, Self::Error> {
        unimplemented!()
    }

    /*fn from_form_value(form_value: &'v RawStr) -> Result<NumberVec, &'v RawStr> {
        let mut str = form_value.to_string();
        str = str
            .replace("[", "")
            .replace("]", "")
            .replace(" ", "")
            .replace('\n', "");
        let raw_ids = str.split(",").collect::<Vec<&str>>();
        let ids: Result<Vec<_>, _> = raw_ids.iter().map(parse_str_to_u64).collect();
        match ids {
            Ok(ids) => Ok(NumberVec(ids)),
            _ => Err(RawStr::from_str(
                "Integer array contains unsupported values",
            )),
        }
    }*/
}

// See: https://stackoverflow.com/a/26370894/12347616
// use std::num::ParseIntError;
// fn parse_str_to_u64(input: &&str) -> Result<u64, ParseIntError> {
//     input.parse::<u64>()
// }
//
// // See: https://stackoverflow.com/a/38447886/12347616
// fn crop_letters(s: &str, pos: usize) -> String {
//     match s.char_indices().skip(pos).next() {
//         Some((pos, _)) => String::from(&s[pos..]),
//         None => "".to_string(),
//     }
// }

impl<'v> FromFormValue<'v> for RouteCriterionVec {
    type Error = &'v RawStr;

    /*
    fn from_form_value(form_value: &'v RawStr) -> Result<CriterionVec, &'v RawStr> {
        let mut str = form_value.to_string();
        str = str
            .replace("[", "")
            .replace("]", "")
            .replace('\n', "")
            .replace("{", "");
        let raw_criteria = str.split("},").collect::<Vec<&str>>();
        let mut criteria: Vec<Criterion> = Vec::new();
        for raw_criterion in raw_criteria {
            let raw_criterion = raw_criterion.to_string().replace("{", "");
            let props = raw_criterion.split(",").collect::<Vec<&str>>();
            let mut kind: Option<Kind> = None;
            let mut title: Option<String> = None;
            let mut content: Option<String> = None;
            let mut weight: Option<f64> = None;
            for prop in props {
                if prop.starts_with("type") {
                    let prop = crop_letters(prop, 5).trim().to_string();
                    match Kind::from(&prop) {
                        Ok(newKind) => kind = Some(newKind),
                        Err(_) => {}
                    }
                } else if prop.starts_with("title") {
                    let prop = crop_letters(prop, 6).trim().to_string();
                    title = Some(prop);
                } else if prop.starts_with("content") {
                    let prop = crop_letters(prop, 10).trim().to_string();
                    content = Some(prop);
                } else if prop.starts_with("weight") {
                    let prop = crop_letters(prop, 8).trim().to_string();
                    match prop.parse::<f64>() {
                        Ok(newWeight) => weight = Some(newWeight),
                        Err(_) => {}
                    }
                }
            }
            if kind.is_some() && title.is_some() && content.is_some() && weight.is_some() {
                criteria.push(Criterion {
                    title: title.unwrap(),
                    content: content.unwrap(),
                    weight: weight.unwrap(),
                    kind: kind.unwrap(),
                });
            } else {
                return Err(RawStr::from_str(
                    "Criteria array contains unsupported or malformed criteria ",
                ));
            }
        }

        Ok(CriterionVec(criteria))
    }*/

    // It seems that a dummy implementation is sufficient?
    // Serde is triggered internally?
    fn from_form_value(_form_value: &'v RawStr) -> Result<RouteCriterionVec, &'v RawStr> {
        unimplemented!()
    }
}
