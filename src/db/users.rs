use crate::models::{NewStudent, User};
use crate::schema::users::dsl::{
    firstname as dsl_firstname, id as dsl_id, lastname as dsl_lastname, username as dsl_username,
    users,
};
use diesel::prelude::*;
use diesel::result::Error;

pub fn create_user<'a>(
    conn: &MysqlConnection,
    username: String,
    firstname: String,
    lastname: String,
    password: String,
) -> Result<User, &'static str> {
    let exists: Result<User, _> = users.filter(dsl_username.eq(&username)).first(conn);
    if exists.is_ok() {
        return Err("Already exists");
    }

    // TODO get unit from params?
    let unit = String::from("3A2");
    let new_user = NewStudent::new(username, firstname, lastname, password, unit);

    diesel::insert_into(users)
        .values(&new_user)
        .execute(conn)
        .expect("Error saving new user");

    Ok(users.order(dsl_id.desc()).first(conn).unwrap())
}

pub fn get_by_id(conn: &MysqlConnection, id: u64) -> Result<User, Error> {
    users.filter(dsl_id.eq(id)).first(conn)
}

pub fn get_by_name(conn: &MysqlConnection, username: &str) -> Result<User, Error> {
    users.filter(dsl_username.eq(username)).first(conn)
}

pub fn get_by_firstname_lastname(
    conn: &MysqlConnection,
    firstname: &str,
    lastname: &str,
) -> Result<User, Error> {
    // Make query with multiple WHERE statements
    users
        .filter(dsl_lastname.eq(lastname).and(dsl_firstname.eq(firstname)))
        .first(conn)
}
