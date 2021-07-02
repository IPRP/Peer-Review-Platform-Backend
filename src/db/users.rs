//! CRUD operations for users.

use crate::models::{NewStudent, NewTeacher, Role, User};
use crate::schema::users::dsl::{
    firstname as dsl_firstname, id as dsl_id, lastname as dsl_lastname, role as dls_role,
    unit as dsl_unit, username as dsl_username, users,
};
use diesel::prelude::*;
use diesel::result::Error;

/// Create student account.
pub fn create_student<'a>(
    conn: &MysqlConnection,
    username: String,
    firstname: String,
    lastname: String,
    password: String,
    unit: String,
) -> Result<User, &'static str> {
    let exists: Result<User, _> = users.filter(dsl_username.eq(&username)).first(conn);
    if exists.is_ok() {
        return Err("Already exists");
    }
    let new_user = NewStudent::new(username, firstname, lastname, password, unit);

    diesel::insert_into(users)
        .values(&new_user)
        .execute(conn)
        .expect("Error saving new user");

    Ok(users.order(dsl_id.desc()).first(conn).unwrap())
}

/// Create teacher account.
pub fn create_teacher<'a>(
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
    let new_user = NewTeacher::new(username, firstname, lastname, password);

    diesel::insert_into(users)
        .values(&new_user)
        .execute(conn)
        .expect("Error saving new user");

    Ok(users.order(dsl_id.desc()).first(conn).unwrap())
}

/// Get user by user id.
pub fn get_by_id(conn: &MysqlConnection, id: u64) -> Result<User, Error> {
    users.filter(dsl_id.eq(id)).first(conn)
}

/// Get user by username.
pub fn get_by_name(conn: &MysqlConnection, username: &str) -> Result<User, Error> {
    users.filter(dsl_username.eq(username)).first(conn)
}

/// Simplified representation of an user.
#[derive(Serialize, Clone)]
pub struct SimpleUser {
    pub id: u64,
    pub firstname: String,
    pub lastname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "group")]
    pub unit: Option<String>,
}

/// Get all students in simplified representation.
pub fn get_all_students(conn: &MysqlConnection) -> Result<Vec<SimpleUser>, ()> {
    let students = users
        .filter(dls_role.eq(Role::Student))
        .select((dsl_id, dsl_firstname, dsl_lastname, dsl_unit))
        .get_results::<(u64, String, String, Option<String>)>(conn);
    if students.is_err() {
        return Err(());
    }
    let students: Vec<(u64, String, String, Option<String>)> = students.unwrap();
    Ok(students
        .into_iter()
        .map(|user| SimpleUser {
            id: user.0,
            firstname: user.1,
            lastname: user.2,
            unit: user.3,
        })
        .collect())
}

/// Get student by student id.
pub fn get_student_by_id(conn: &MysqlConnection, id: u64) -> Result<User, Error> {
    // Make query with multiple WHERE statements
    users
        .filter(dsl_id.eq(id).and(dls_role.eq(Role::Student)))
        .first(conn)
}

/// Get student by firstname & lastname.
pub fn get_student_by_firstname_lastname(
    conn: &MysqlConnection,
    firstname: &str,
    lastname: &str,
) -> Result<User, Error> {
    // Make query with multiple WHERE statements
    users
        .filter(
            dsl_lastname
                .eq(lastname)
                .and(dsl_firstname.eq(firstname))
                .and(dls_role.eq(Role::Student)),
        )
        .first(conn)
}

/// Get all student ids by unit (group).
pub fn get_students_by_unit(conn: &MysqlConnection, unit: &str) -> Result<Vec<User>, Error> {
    users
        .filter(dsl_unit.eq(unit).and(dls_role.eq(Role::Student)))
        .get_results::<User>(conn)
}
