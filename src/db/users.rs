use crate::models::{NewUser, User};
use crate::schema::users::dsl::{id, username as dsl_username, users};
use diesel::prelude::*;
use diesel::result::Error;

pub fn create_user<'a>(
    conn: &MysqlConnection,
    username: String,
    password: String,
) -> Result<User, &'static str> {
    let exists: Result<User, _> = users.filter(dsl_username.eq(&username)).first(conn);
    if exists.is_ok() {
        return Err("Already exists");
    }

    let new_user = NewUser { username, password };

    diesel::insert_into(users)
        .values(&new_user)
        .execute(conn)
        .expect("Error saving new user");

    Ok(users.order(id.desc()).first(conn).unwrap())
}

pub fn get_user(conn: &MysqlConnection, username: &str) -> Result<User, Error> {
    users.filter(dsl_username.eq(username)).first(conn)
}
