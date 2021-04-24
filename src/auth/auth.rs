use crate::models::User;
use crate::IprpDB;
use base64::DecodeError;
use diesel::result::Error;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use std::string::FromUtf8Error;

pub struct AuthenticatedUser {
    pub(crate) user_id: u64,
}

#[derive(Debug)]
pub enum LoginError {
    InvalidData,
    UsernameDoesNotExist,
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
                                let user = crate::db::users::get_user(&*conn, &u);
                                match user {
                                    Ok(user) => {
                                        let hashed_password =
                                            crate::auth::crypto::hash_password(&p.to_string());
                                        if user.password == hashed_password {
                                            Ok(user.id)
                                        } else {
                                            Err("Mismatched passwords")
                                        }
                                    }
                                    Err(_) => Err("No such user"),
                                }
                            }
                        });
                    match auth_result {
                        Ok(id) => Outcome::Success(AuthenticatedUser { user_id: *id }),
                        Err(_) => Outcome::Failure((Status::BadRequest, LoginError::InvalidData)),
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
            Outcome::Success(AuthenticatedUser { user_id })
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
                let mut auth_split = auth.splitn(2, ":").collect::<Vec<&str>>();
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
