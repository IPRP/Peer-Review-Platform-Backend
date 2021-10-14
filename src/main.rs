#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_cors;
extern crate rocket_multipart_form_data;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel_derive_enum;
#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate crypto;

use rocket::fairing::AdHoc;
use rocket_cors::{CorsOptions};

// Import database operations
mod db;
// Import defined model structs
mod models;
// Import generated schemas
mod schema;
// import routes
mod routes;
// import auth handler
mod auth;
// import cors handler
mod cors;
// import utilities
mod utils;

// Configure Database
#[database("iprp_db")]
pub struct IprpDB(diesel::MysqlConnection);

// Start
fn main() {
    // Setup cors
    let cors = CorsOptions {
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .unwrap();
    // Launch Rocket
    rocket::ignite()
        .attach(IprpDB::fairing())
        .attach(AdHoc::on_attach("Database Migration", db::run_db_migration))
        .attach(AdHoc::on_attach(
            "Review Configuration",
            db::setup_review_timespan,
        ))
        .attach(cors)
        .mount(
            "/",
            routes![
                routes::users::login,
                routes::users::logout,
                routes::users::create_student,
                routes::users::create_teacher,
                routes::teachers::workshop,
                routes::teachers::workshops,
                routes::teachers::search_student,
                routes::teachers::create_workshop,
                routes::teachers::update_workshop,
                routes::teachers::delete_workshop,
                routes::attachments::upload,
                routes::attachments::download,
                routes::attachments::remove,
                routes::students::workshop,
                routes::students::workshops,
                routes::students::todos,
                routes::submissions::create_submission,
                routes::submissions::get_submission,
                routes::submissions::update_review,
                routes::submissions::get_review,
            ],
        )
        .launch();
}
