use crate::models::{NewStudent, NewWorkshop, Role, User, Workshop};
use crate::schema::workshops::dsl::{
    anonymous as dsl_anonymous, content as dsl_content, end as dls_end, id as dsl_id,
    title as dsl_title, workshops,
};
use diesel::prelude::*;
use diesel::result::Error;

pub fn create_workshop<'a>(
    conn: &MysqlConnection,
    title: String,
    content: String,
    end: chrono::NaiveDate,
    anonymous: bool,
) -> Result<Workshop, &'static str> {
    let new_workshop = NewWorkshop {
        title,
        content,
        end,
        anonymous,
    };

    diesel::insert_into(workshops)
        .values(&new_workshop)
        .execute(conn)
        .expect("Error saving new workshop");

    Ok(workshops.order(dsl_id.desc()).first(conn).unwrap())
}
