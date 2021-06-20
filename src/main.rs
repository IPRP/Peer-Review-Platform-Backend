#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
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
    rocket::ignite()
        .attach(IprpDB::fairing())
        .attach(AdHoc::on_attach("Database Migration", db::run_db_migration))
        .attach(cors::CORS)
        .mount(
            "/",
            routes![
                routes::test::index,
                routes::test::image,
                routes::test::form,
                routes::test::auth_test,
                routes::test::login,
                routes::test::logout,
                routes::db::create_user,
                routes::teachers::workshops,
                routes::teachers::search_student,
                routes::teachers::create_workshop,
                routes::teachers::delete_workshop,
                routes::attachments::upload,
                routes::attachments::download,
                routes::attachments::remove,
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
