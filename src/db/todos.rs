use crate::db;
use crate::schema::reviews::dsl::{
    deadline as review_deadline, done as review_done, id as review_id, reviewer,
    reviews as reviews_t, submission as review_submission,
};
use crate::schema::submissions::dsl::{
    id as sub_id, student as sub_student, submissions as submissions_t, title as sub_title,
    workshop as sub_ws,
};
use crate::schema::users::dsl::{
    firstname as user_firstname, id as user_id, lastname as user_lastname, users as users_t,
};
use crate::schema::workshops::dsl::{id as ws_id, title as ws_title, workshops as workshops_t};
use diesel::prelude::*;
use diesel::result::Error;

#[derive(Deserialize)]
pub struct TodoReview {
    id: u64,
    done: bool,
    deadline: chrono::NaiveDateTime,
    title: String,
    firstname: String,
    lastname: String,
    submission: u64,
    #[serde(rename(serialize = "workshopName"))]
    workshop_name: u64,
}

#[derive(Deserialize)]
pub struct TodoSubmission {
    id: u64,
    #[serde(rename(serialize = "workshopName"))]
    workshop_name: u64,
}

#[derive(Deserialize)]
pub struct Todo {
    reviews: Vec<TodoReview>,
    submissions: Vec<TodoSubmission>,
}

pub fn get(conn: &MysqlConnection, student_id: u64) -> Result<(), ()> {
    // TODO filter by deadline
    // TODO remove name if workshop is anonymous
    let reviews = reviews_t
        .inner_join(submissions_t.on(sub_id.eq(review_submission)))
        .inner_join(workshops_t.on(ws_id.eq(sub_ws)))
        .inner_join(users_t.on(user_id.nullable().eq(sub_student.nullable())))
        .filter(reviewer.eq(student_id))
        .select((
            review_id,
            review_done,
            review_deadline,
            sub_title,
            user_firstname,
            user_lastname,
            ws_title,
        ))
        .get_results::<(
            u64,
            bool,
            chrono::NaiveDateTime,
            String,
            String,
            String,
            String,
        )>(conn);

    // Iterate + map

    /*
    select r.id, r.done, r.deadline, s.id, u.firstname, u.lastname, w.title
         from reviews r
         inner join submissions s on r.submission=s.id
         inner join workshops w on s.workshop=w.id
         inner join users u on s.student=u.id
         where s.student=4;
     */

    Ok(())
}
