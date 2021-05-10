use crate::schema::users;

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

#[derive(Queryable)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub password: String,
    pub role: String,
    pub unit: Option<String>,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewStudent {
    pub username: String,
    pub password: String,
    pub role: String,
    pub unit: String,
}

impl NewStudent {
    pub fn new(username: String, password: String, unit: String) -> Self {
        let role = String::from("student");
        NewStudent {
            username,
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
    pub password: String,
    pub role: String,
}

impl NewTeacher {
    pub fn new(username: String, password: String) -> Self {
        let role = String::from("teacher");
        NewTeacher {
            username,
            password,
            role,
        }
    }
}
