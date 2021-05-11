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

// TODO patch file (MySQL Enum to RoleMapping)
// See: http://diesel.rs/guides/configuring-diesel-cli.html
// And: https://github.com/adwhit/diesel-derive-enum/issues/56
// And: https://github.com/diesel-rs/diesel/issues/2154

#[derive(DbEnum, Clone, Debug, PartialEq)]
pub enum Role {
    Student,
    Teacher,
}

#[derive(Queryable, Clone)]
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
