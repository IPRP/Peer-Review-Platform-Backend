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
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub password: String,
}
