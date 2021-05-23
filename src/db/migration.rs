use crate::diesel::connection::SimpleConnection;
use crate::IprpDB;
use rocket::logger::error;
use rocket::Rocket;

// Perform migrations automatically without CLI
// Based on https://stackoverflow.com/a/61064269/12347616
embed_migrations!();
pub fn run_db_migration(rocket: Rocket) -> Result<Rocket, Rocket> {
    let conn = IprpDB::get_one(&rocket).expect("database connection");
    match embedded_migrations::run(&*conn) {
        Ok(()) => {
            // TODO remove this later on or only run db reset on custom flag
            // Note:
            // fb001dfcffd1c899f3297871406242f097aecf1a5342ccf3ebcd116146188e4b
            // => admin
            // 1d6442ddcfd9db1ff81df77cbefcd5afcc8c7ca952ab3101ede17a84b866d3f3
            // => 1234
            // --
            // Truncate tables with foreign keys
            // See: https://stackoverflow.com/a/5452798/12347616
            let res = conn.batch_execute(
                r#"
SET FOREIGN_KEY_CHECKS = 0;
truncate criteria;
truncate criterion;
truncate users;
truncate workshoplist;
truncate workshops;
truncate attachments;
truncate submissions;
truncate submissioncriteria;
truncate submissionattachments;
truncate reviews;
truncate reviewpoints;
SET FOREIGN_KEY_CHECKS = 1;
INSERT INTO users values(default, "admin", "admin", "admin", "fb001dfcffd1c899f3297871406242f097aecf1a5342ccf3ebcd116146188e4b", "teacher", null);
INSERT INTO users values(default, "t1", "John", "Doe", "1d6442ddcfd9db1ff81df77cbefcd5afcc8c7ca952ab3101ede17a84b866d3f3", "teacher", null);
INSERT INTO users values(default, "t2", "John", "Doe II", "1d6442ddcfd9db1ff81df77cbefcd5afcc8c7ca952ab3101ede17a84b866d3f3", "teacher", null);
INSERT INTO users values(default, "s1", "Max", "Mustermann", "1d6442ddcfd9db1ff81df77cbefcd5afcc8c7ca952ab3101ede17a84b866d3f3", "student", "4A");
INSERT INTO users values(default, "s2", "Luke", "Skywalker", "1d6442ddcfd9db1ff81df77cbefcd5afcc8c7ca952ab3101ede17a84b866d3f3", "student", "4A");
INSERT INTO users values(default, "s3", "Gordon", "Freeman", "1d6442ddcfd9db1ff81df77cbefcd5afcc8c7ca952ab3101ede17a84b866d3f3", "student", "4A");
INSERT INTO users values(default, "s4", "Mario", "Mario", "1d6442ddcfd9db1ff81df77cbefcd5afcc8c7ca952ab3101ede17a84b866d3f3", "student", "4A");
    "#,
            );
            println!("Database reset: {:?}", res);
            Ok(rocket)
        }
        Err(e) => {
            error(&format!("Failed to run database migrations: {:?}", e));
            Err(rocket)
        }
    }
}
