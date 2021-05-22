use crate::schema::*;

/**
#[derive(Queryable)]
pub struct Post {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(Insertable)]
#[table_name = "posts"]
pub struct NewPost<'a> {
    pub title: &'a str,
    pub body: &'a str,
}*/

// TODO patch file (MySQL Enum to RoleMapping)
// See: http://diesel.rs/guides/configuring-diesel-cli.html
// And: https://github.com/adwhit/diesel-derive-enum/issues/56
// And: https://github.com/diesel-rs/diesel/issues/2154

#[derive(DbEnum, Clone, Debug, PartialEq)]
pub enum Role {
    Student,
    Teacher,
}

impl Role {
    pub fn to_string(&self) -> String {
        match self {
            Role::Student => String::from("student"),
            Role::Teacher => String::from("teacher"),
        }
    }
}

#[derive(Debug, Queryable, Clone)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub password: String,
    pub role: Role,
    pub unit: Option<String>,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewStudent {
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub password: String,
    pub role: Role,
    pub unit: String,
}

impl NewStudent {
    pub fn new(
        username: String,
        firstname: String,
        lastname: String,
        password: String,
        unit: String,
    ) -> Self {
        let role = Role::Student;
        NewStudent {
            username,
            firstname,
            lastname,
            password,
            role,
            unit,
        }
    }
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewTeacher {
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub password: String,
    pub role: Role,
}

impl NewTeacher {
    pub fn new(username: String, firstname: String, lastname: String, password: String) -> Self {
        let role = Role::Teacher;
        NewTeacher {
            username,
            firstname,
            lastname,
            password,
            role,
        }
    }
}

#[derive(Queryable, Clone)]
pub struct Workshop {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub end: chrono::NaiveDate,
    pub anonymous: bool,
}

#[derive(Insertable)]
#[table_name = "workshops"]
pub struct NewWorkshop {
    pub title: String,
    pub content: String,
    pub end: chrono::NaiveDate,
    pub anonymous: bool,
}

#[derive(Insertable, Queryable, Clone)]
#[table_name = "criteria"]
pub struct Criteria {
    pub workshop: u64,
    pub criterion: u64,
}

#[derive(DbEnum, Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Point,
    Grade,
    Percentage,
    Truefalse,
}

impl Kind {
    pub fn from(str: &str) -> Result<Self, String> {
        let input = str.to_lowercase();
        if input.eq("point") {
            Ok(Kind::Point)
        } else if input.eq("grade") {
            Ok(Kind::Grade)
        } else if input.eq("percentage") {
            Ok(Kind::Percentage)
        } else if input.eq("truefalse") {
            Ok(Kind::Truefalse)
        } else {
            Err(String::new())
        }
    }
}

#[derive(Queryable, Clone)]
pub struct Criterion {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub weight: f64,
    pub kind: Kind,
}

#[derive(Insertable)]
#[table_name = "criterion"]
pub struct NewCriterion {
    pub title: String,
    pub content: String,
    pub weight: f64,
    pub kind: Kind,
}

#[derive(Insertable, Queryable, Clone)]
#[table_name = "workshoplist"]
pub struct Workshoplist {
    pub workshop: u64,
    pub user: u64,
    pub role: Role,
}
