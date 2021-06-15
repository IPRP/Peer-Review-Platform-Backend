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

#[derive(Serialize)]
pub struct TodoReview {
    pub id: u64,
    pub done: bool,
    pub deadline: chrono::NaiveDateTime,
    pub title: String,
    pub firstname: String,
    pub lastname: String,
    pub submission: u64,
    #[serde(rename(serialize = "workshopName"))]
    pub workshop_name: String,
}

#[derive(Serialize)]
pub struct TodoSubmission {
    pub id: u64,
    #[serde(rename(serialize = "workshopName"))]
    pub workshop_name: u64,
}

#[derive(Serialize)]
pub struct Todo {
    pub reviews: Vec<TodoReview>,
    pub submissions: Vec<TodoSubmission>,
}

pub fn get(conn: &MysqlConnection, student_id: u64) -> Result<Todo, ()> {
    // TODO filter by deadline
    // TODO remove name if workshop is anonymous

    /*
    select r.id, r.done, r.deadline, s.id, u.firstname, u.lastname, w.title
         from reviews r
         inner join submissions s on r.submission=s.id
         inner join workshops w on s.workshop=w.id
         inner join users u on s.student=u.id
         where s.student=4;
     */

    let raw_reviews = reviews_t
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
            sub_id,
            ws_title,
        ))
        .get_results::<(
            u64,
            bool,
            chrono::NaiveDateTime,
            String,
            String,
            String,
            u64,
            String,
        )>(conn);

    if raw_reviews.is_err() {
        return Err(());
    }
    let raw_reviews = raw_reviews.unwrap();

    let reviews: Vec<TodoReview> = raw_reviews
        .into_iter()
        .map(|review| TodoReview {
            id: review.0,
            done: review.1,
            deadline: review.2,
            title: review.3,
            firstname: review.4,
            lastname: review.5,
            submission: review.6,
            workshop_name: review.7,
        })
        .collect();

    // TODO submissions

    Ok(Todo {
        reviews,
        submissions: vec![],
    })
}