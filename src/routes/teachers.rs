use crate::models::{Kind, NewCriterion, Role, User};
use crate::routes::models::ApiResponse;
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

#[derive(FromForm, Deserialize)]
pub struct SearchStudent {
    id: Option<u64>,
    firstname: Option<String>,
    lastname: Option<String>,
    group: Option<String>,
}

#[get("/teachers/search/student", format = "json", data = "<search_info>")]
pub fn search_student(
    user: User,
    conn: IprpDB,
    search_info: Json<SearchStudent>,
) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Student {
        return Err(ApiResponse::forbidden());
    }

    if search_info.id.is_some() {
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

// You can use this deserializer for any type that implements FromStr
// and the FromStr::Err implements Display
fn deserialize_from_str<'de, S, D>(deserializer: D) -> Result<S, D::Error>
where
    S: FromStr,      // Required for S::from_str...
    S::Err: Display, // Required for .map_err(de::Error::custom)
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    println!("{} s", &s);
    S::from_str(&s).map_err(de::Error::custom)
}

// Expected format is ISO 8601 combined date and time without timezone like `2007-04-05T14:30:30`
// In JS that would mean creating a Date like this `(new Date()).toISOString().split(".")[0]`
// https://github.com/serde-rs/json/issues/531#issuecomment-479738561
#[derive(Deserialize)]
pub struct Date(chrono::NaiveDateTime);

#[derive(Deserialize)]
pub struct NumberVec(Vec<u64>);

#[derive(Debug, Deserialize)]
pub struct Criterion {
    title: String,
    content: String,
    weight: f64,
    #[serde(rename = "type")]
    kind: Kind,
}

impl From<Criterion> for NewCriterion {
    fn from(item: Criterion) -> Self {
        NewCriterion {
            title: item.title,
            content: item.content,
            weight: item.weight,
            kind: item.kind,
        }
    }
}

impl From<CriterionVec> for Vec<NewCriterion> {
    fn from(items: CriterionVec) -> Self {
        items
            .0
            .into_iter()
            .map(|item| NewCriterion::from(item))
            .collect()
    }
}

impl From<NumberVec> for Vec<u64> {
    fn from(items: NumberVec) -> Self {
        items.0
    }
}

#[derive(Deserialize)]
pub struct CriterionVec(Vec<Criterion>);

#[derive(FromForm, Deserialize)]
pub struct NewWorkshop {
    title: String,
    content: String,
    end: Date,
    anonymous: bool,
    teachers: NumberVec,
    students: NumberVec,
    criteria: CriterionVec,
}

#[post("/teachers/workshop", format = "json", data = "<new_workshop>")]
pub fn create_workshop(
    user: User,
    conn: IprpDB,
    new_workshop: Json<NewWorkshop>,
) -> Result<Json<JsonValue>, ApiResponse> {
    if user.role == Role::Student {
        return Err(ApiResponse::forbidden());
    }

    println!("{:?}", new_workshop.end.0);
    println!("{:?}", new_workshop.students.0);
    println!("{:?}", new_workshop.criteria.0);

    let workshop = db::workshops::create(
        &*conn,
        new_workshop.0.title,
        new_workshop.0.content,
        new_workshop.0.end.0,
        new_workshop.0.anonymous,
        Vec::from(new_workshop.0.teachers),
        Vec::from(new_workshop.0.students),
        Vec::from(new_workshop.0.criteria),
    );
    match workshop {
        Ok(workshop) => Ok(Json(json!({
            "ok": true,
            "id": workshop.id
        }))),
        Err(_) => Err(ApiResponse::conflict()),
    }
}

#[delete("/teachers/workshop/<id>")]
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

// See: https://api.rocket.rs/v0.4/rocket/request/trait.FromFormValue.html#example
impl<'v> FromFormValue<'v> for Date {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
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

// See: https://stackoverflow.com/a/26370894/12347616
fn parse_str_to_u64(input: &&str) -> Result<u64, ParseIntError> {
    input.parse::<u64>()
}

impl<'v> FromFormValue<'v> for NumberVec {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
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

// See: https://stackoverflow.com/a/38447886/12347616
fn crop_letters(s: &str, pos: usize) -> String {
    match s.char_indices().skip(pos).next() {
        Some((pos, _)) => String::from(&s[pos..]),
        None => "".to_string(),
    }
}

impl<'v> FromFormValue<'v> for CriterionVec {
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
    fn from_form_value(form_value: &'v RawStr) -> Result<CriterionVec, &'v RawStr> {
        unimplemented!()
    }
}
