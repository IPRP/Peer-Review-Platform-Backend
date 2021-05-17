use crate::models::{Kind, Role, User};
use crate::{db, IprpDB};
use rocket::http::{RawStr, Status};
use rocket::request::FromFormValue;
use rocket::response::content;
use rocket_contrib::json;
use std::num::{ParseFloatError, ParseIntError};

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

#[derive(Deserialize)]
pub struct Date(chrono::NaiveDate);

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
    _user: User,
    conn: IprpDB,
    new_workshop: json::Json<NewWorkshop>,
) -> Result<content::Json<String>, Status> {
    println!("{:?}", new_workshop.students.0);
    println!("{:?}", new_workshop.criteria.0);
    Err(Status::ImATeapot)
}

// See: https://api.rocket.rs/v0.4/rocket/request/trait.FromFormValue.html#example
impl<'v> FromFormValue<'v> for Date {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<Date, &'v RawStr> {
        // See: https://docs.rs/chrono/0.4.19/chrono/naive/struct.NaiveDate.html#method.parse_from_str
        let date = chrono::NaiveDate::parse_from_str("2015-09-05", "%Y-%m-%d");
        match date {
            Ok(date) => Ok(Date(date)),
            _ => Err(RawStr::from_str(
                "Date should be formatted `%Y-%m-%d` like `2015-09-05`",
            )),
        }
    }
}

// See: https://stackoverflow.com/a/26370894/12347616
fn parse_str_to_u64(input: &&str) -> Result<u64, ParseIntError> {
    input.parse::<u64>()
}

impl<'v> FromFormValue<'v> for NumberVec {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<NumberVec, &'v RawStr> {
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
    }
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
        Ok(CriterionVec(Vec::new()))
    }
}
