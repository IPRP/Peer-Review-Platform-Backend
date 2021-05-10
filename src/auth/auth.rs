use crate::models::User;
use crate::IprpDB;
use base64::DecodeError;
use diesel::result::Error;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use std::string::FromUtf8Error;

#[derive(Copy, Clone)]
pub struct AuthenticatedUser {
    pub(crate) user_id: u64,
    pub(crate) role: Role,
}

#[derive(Copy, Clone)]
pub enum Role {
    Student,
    Teacher,
}

impl Role {
    fn determine(role: String) -> Role {
        if role == "teacher" {
            Role::Teacher
        } else {
            Role::Student
        }
    }
}

#[derive(Debug)]
pub enum LoginError {
    InvalidData,
    UserDoesNotExist,
    WrongPassword,
}

/// Handles authentication.
/// Is invoked when an endpoint contains `AuthenticatedUser` as parameter.
/// Supports Basic Authorization and Authorization via Cookie.
/// For Authorization via Cookie the endpoint `/users/login` needs to be invoked first.
impl<'a, 'r> FromRequest<'a, 'r> for AuthenticatedUser {
    type Error = LoginError;
    fn from_request(request: &'a Request<'r>) -> Outcome<AuthenticatedUser, LoginError> {
        // Basic Auth
        if let Some(auth_header) = request.headers().get_one("authorization") {
            match get_basic_auth_info(auth_header) {
                Ok(out) => {
                    let (u, p) = out;
                    // See: https://api.rocket.rs/v0.4/rocket/request/trait.FromRequest.html#request-local-state
                    let auth_result =
                        request.local_cache(|| match request.guard::<IprpDB>().succeeded() {
                            None => Err("No db connection"),
                            Some(conn) => {
                                let user = crate::db::users::get_by_name(&*conn, &u);
                                match user {
                                    Ok(user) => {
                                        let hashed_password =
                                            crate::auth::crypto::hash_password(&p.to_string());
                                        if user.password == hashed_password {
                                            let user_id = user.id;
                                            //let username = user.username;
                                            let role = Role::determine(user.role);
                                            //let unit = user.unit;
                                            let auth_user = AuthenticatedUser { user_id, role };
                                            Ok(auth_user)
                                        } else {
                                            Err("Mismatched passwords")
                                        }
                                    }
                                    Err(_) => Err("No such user"),
                                }
                            }
                        });
                    match auth_result {
                        Ok(auth_user) => Outcome::Success(*auth_user),
                        Err(_) => Outcome::Failure((Status::BadRequest, LoginError::WrongPassword)),
                    }
                }
                Err(_) => {
                    return Outcome::Failure((Status::BadRequest, LoginError::InvalidData));
                }
            }
        }
        // Auth via Cookie
        else if let Some(cookie) = request.cookies().get_private("user_id") {
            let user_id = cookie.value().parse::<u64>().unwrap();
            let auth_result = request.local_cache(|| match request.guard::<IprpDB>().succeeded() {
                None => Err("No db connection"),
                Some(conn) => {
                    let user = crate::db::users::get_by_id(&*conn, user_id);
                    match user {
                        Ok(user) => {
                            //let username = user.username;
                            let role = Role::determine(user.role);
                            //let unit = user.unit;
                            let auth_user = AuthenticatedUser { user_id, role };
                            Ok(auth_user)
                        }
                        Err(_) => Err("No such user"),
                    }
                }
            });
            match auth_result {
                Ok(auth_user) => Outcome::Success(*auth_user),
                Err(_) => Outcome::Failure((Status::BadRequest, LoginError::UserDoesNotExist)),
            }
        }
        // Bad request
        else {
            Outcome::Failure((Status::BadRequest, LoginError::InvalidData))
        }
    }
}

fn get_basic_auth_info(input: &str) -> Result<(String, String), &'static str> {
    let input = input
        .replace("basic", "")
        .replace("Basic", "")
        .replace(" ", "");
    match base64::decode(input) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(auth) => {
                // Split at first `:`
                // See: https://stackoverflow.com/a/11612931/12347616
                // And: https://stackoverflow.com/a/41517340/12347616
                let auth_split = auth.splitn(2, ":").collect::<Vec<&str>>();
                if auth_split.len() == 2 {
                    Ok((
                        auth_split.get(0).unwrap().to_string(),
                        auth_split.get(1).unwrap().to_string(),
                    ))
                } else {
                    Err("Auth Error")
                }
            }
            Err(_) => Err("Conversion Error"),
        },
        Err(_) => Err("Decode Error"),
    }
}
