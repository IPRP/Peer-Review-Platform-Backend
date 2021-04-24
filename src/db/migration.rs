use crate::IprpDB;
use rocket::logger::error;
use rocket::Rocket;

// Perform migrations automatically without CLI
// Based on https://stackoverflow.com/a/61064269/12347616
embed_migrations!();
pub fn run_db_migration(rocket: Rocket) -> Result<Rocket, Rocket> {
    let conn = IprpDB::get_one(&rocket).expect("database connection");
    match embedded_migrations::run(&*conn) {
        Ok(()) => Ok(rocket),
        Err(e) => {
            error(&format!("Failed to run database migrations: {:?}", e));
            Err(rocket)
        }
    }
}
