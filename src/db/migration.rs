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
            // "Clear" db
            // Truncate tables with foreign keys
            // See: https://stackoverflow.com/a/5452798/12347616
            let db_clear = rocket.config().get_bool("db_clear").unwrap_or(false);
            if db_clear {
                let res = conn.batch_execute(
                    r#"
select concat('drop event if exists ', event_name, ';') from information_schema.events;
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
                    "#,
                );
                match res {
                    Err(e) => {
                        error(&format!("Failed to run database clear: {:?}", e));
                        return Err(rocket);
                    }
                    _ => {}
                }
            }

            // Insert admin user
            // See: https://tableplus.com/blog/2018/11/how-to-insert-if-not-exist-mysql.html
            let res = conn.batch_execute(
                r#"          
INSERT IGNORE INTO users values(default, "admin", "admin", "admin", "fb001dfcffd1c899f3297871406242f097aecf1a5342ccf3ebcd116146188e4b", "teacher", null);
                "#,
            );
            match res {
                Err(e) => {
                    error(&format!("Failed to run database admin insert: {:?}", e));
                    return Err(rocket);
                }
                _ => {}
            }

            // Insert mock data
            // ---
            // Note:
            // fb001dfcffd1c899f3297871406242f097aecf1a5342ccf3ebcd116146188e4b
            // => admin
            // 1d6442ddcfd9db1ff81df77cbefcd5afcc8c7ca952ab3101ede17a84b866d3f3
            // => 1234
            let db_mock = rocket
                .config()
                .get_bool("db_insert_mock_data")
                .unwrap_or(false);
            if db_mock {
                let res = conn.batch_execute(
                    r#"
INSERT INTO users values(default, "t1", "John", "Doe", "1d6442ddcfd9db1ff81df77cbefcd5afcc8c7ca952ab3101ede17a84b866d3f3", "teacher", null);
INSERT INTO users values(default, "t2", "John", "Doe II", "1d6442ddcfd9db1ff81df77cbefcd5afcc8c7ca952ab3101ede17a84b866d3f3", "teacher", null);
INSERT INTO users values(default, "s1", "Max", "Mustermann", "1d6442ddcfd9db1ff81df77cbefcd5afcc8c7ca952ab3101ede17a84b866d3f3", "student", "4A");
INSERT INTO users values(default, "s2", "Luke", "Skywalker", "1d6442ddcfd9db1ff81df77cbefcd5afcc8c7ca952ab3101ede17a84b866d3f3", "student", "4A");
INSERT INTO users values(default, "s3", "Gordon", "Freeman", "1d6442ddcfd9db1ff81df77cbefcd5afcc8c7ca952ab3101ede17a84b866d3f3", "student", "4A");
INSERT INTO users values(default, "s4", "Mario", "Mario", "1d6442ddcfd9db1ff81df77cbefcd5afcc8c7ca952ab3101ede17a84b866d3f3", "student", "4A");
INSERT INTO `workshops` VALUES (1,'WS','Hey!','2021-07-31 16:26:00',1);
INSERT INTO `workshoplist` VALUES (1,1,'teacher'),(1,4,'student'),(1,5,'student');
INSERT INTO `criterion` VALUES (1,'Criterion','True/False',10,'truefalse'),(2,'Other Criterion','True/False',10,'truefalse');
INSERT INTO `criteria` VALUES (1,1),(1,2);
    "#,
                );
                match res {
                    Err(e) => {
                        error(&format!("Failed to run database mock insert: {:?}", e));
                        return Err(rocket);
                    }
                    _ => {}
                }
            }

            println!("Database reset: ");
            println!("    => OK");
            Ok(rocket)
        }
        Err(e) => {
            error(&format!("Failed to run database migrations: {:?}", e));
            Err(rocket)
        }
    }
}
