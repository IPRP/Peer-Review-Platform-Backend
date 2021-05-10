use crate::models::{NewStudent, User};
use crate::schema::users::dsl::{id as dsl_id, username as dsl_username, users};
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

    // TODO get unit from params?
    let unit = String::from("3A2");
    let new_user = NewStudent::new(username, password, unit);

    diesel::insert_into(users)
        .values(&new_user)
        .execute(conn)
        .expect("Error saving new user");

    Ok(users.order(dsl_id.desc()).first(conn).unwrap())
}

pub fn get_by_name(conn: &MysqlConnection, username: &str) -> Result<User, Error> {
    users.filter(dsl_username.eq(username)).first(conn)
}

pub fn get_by_id(conn: &MysqlConnection, id: u64) -> Result<User, Error> {
    users.filter(dsl_id.eq(id)).first(conn)
}
