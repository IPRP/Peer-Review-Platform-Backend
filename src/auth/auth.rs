//! Authentication handling for Rocket.

use crate::models::User;
use crate::IprpDB;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;

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
impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = LoginError;
    fn from_request(request: &'a Request<'r>) -> Outcome<User, LoginError> {
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
                                            Ok(user)
                                        } else {
                                            Err("Mismatched passwords")
                                        }
                                    }
                                    Err(_) => Err("No such user"),
                                }
                            }
                        });
                    match auth_result {
                        Ok(user) => Outcome::Success(user.clone()),
                        Err(_) => {
                            Outcome::Failure((Status::Unauthorized, LoginError::WrongPassword))
                        }
                    }
                }
                Err(_) => {
                    return Outcome::Failure((Status::Unauthorized, LoginError::InvalidData));
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
                        Ok(user) => Ok(user),
                        Err(_) => Err("No such user"),
                    }
                }
            });
            match auth_result {
                Ok(user) => Outcome::Success(user.clone()),
                Err(_) => Outcome::Failure((Status::Unauthorized, LoginError::UserDoesNotExist)),
            }
        }
        // Bad request
        else {
            Outcome::Failure((Status::Unauthorized, LoginError::InvalidData))
        }
    }
}

/// Get username & password from Basic Authentication Header.
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
