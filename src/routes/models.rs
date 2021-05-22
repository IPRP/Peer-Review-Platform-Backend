use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response;
use rocket::response::{Responder, Response};
use rocket_contrib::json::{Json, JsonValue};

// Based on: https://stackoverflow.com/a/54867136/12347616
#[derive(Debug)]
pub struct ApiResponse {
    json: JsonValue,
    status: Status,
}

impl ApiResponse {
    pub fn bad_request() -> Self {
        let json = json!({
            "ok": false
        });
        let status = Status::BadRequest;
        ApiResponse { json, status }
    }

    pub fn conflict() -> Self {
        let json = json!({
            "ok": false
        });
        let status = Status::Conflict;
        ApiResponse { json, status }
    }

    pub fn forbidden() -> Self {
        let json = json!({
            "ok": false
        });
        let status = Status::Forbidden;
        ApiResponse { json, status }
    }

    pub fn not_found() -> Self {
        let json = json!({
            "ok": false
        });
        let status = Status::NotFound;
        ApiResponse { json, status }
    }
}

impl<'r> Responder<'r> for ApiResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}
