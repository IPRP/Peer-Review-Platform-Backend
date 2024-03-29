//! CRUD operations for submissions.

use crate::db;
use crate::db::error::{DbError, DbErrorKind};
use crate::db::models::*;
use crate::db::submissions::calculate_points;
use crate::schema::reviews::dsl::{
    deadline as review_deadline, done as review_done, id as review_id, locked as review_locked,
    reviewer, reviews as reviews_t, submission as review_submission,
};
use crate::schema::submissions::dsl::{
    deadline as sub_deadline, id as sub_id, reviewsdone as sub_reviews_done,
    student as sub_student, submissions as submissions_t, title as sub_title, workshop as sub_ws,
};
use crate::schema::users::dsl::{
    firstname as user_firstname, id as user_id, lastname as user_lastname, users as users_t,
};
use crate::schema::workshoplist::dsl::{
    user as wsl_user, workshop as wsl_ws, workshoplist as workshoplist_t,
};
use crate::schema::workshops::dsl::{
    end as ws_end, id as ws_id, title as ws_title, workshops as workshops_t,
};
use crate::utils::error::AppError;
use chrono::Local;
use diesel::dsl::exists;
use diesel::dsl::not;
use diesel::prelude::*;

/// Get student T O D O.
pub fn get(conn: &MysqlConnection, student_id: u64) -> Result<Todo, DbError> {
    // Query Submission IDs to related Submissions/Reviews where points & reviews
    // were not processed yet
    // --------------------------------------------------------------------------
    let now = Local::now().naive_local();
    let submissions_to_close: QueryResult<Vec<u64>> = reviews_t
        .inner_join(submissions_t.on(sub_id.eq(review_submission)))
        .inner_join(workshops_t.on(ws_id.eq(sub_ws)))
        .inner_join(users_t.on(user_id.nullable().eq(sub_student.nullable())))
        .filter(
            reviewer.eq(student_id).and(
                review_locked
                    .eq(false)
                    .and(sub_reviews_done.eq(false).and(sub_deadline.lt(now))),
            ),
        )
        .select(sub_id)
        .get_results::<u64>(conn);

    if submissions_to_close.is_err() {
        return Err(DbError::new(
            DbErrorKind::ReadFailed,
            "Could not query done Submissions",
        ));
    }
    let submissions_to_close = submissions_to_close.unwrap();
    // Process finished submissions
    for submission_id in submissions_to_close {
        if let Err(source_err) = calculate_points(conn, submission_id) {
            return Err(DbError::new_with_source(
                DbErrorKind::UpdateFailed,
                "Could not process finished Submissions",
                Box::new(source_err) as Box<dyn AppError>,
            ));
        }
    }

    // Query reviews that can still be updated (deadline not reached yet)
    // ------------------------------------------------------------------
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
        .filter(reviewer.eq(student_id).and(review_locked.eq(false)))
        .select((
            review_id,
            review_done,
            review_deadline,
            sub_title,
            user_firstname,
            user_lastname,
            sub_id,
            ws_title,
            ws_id,
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
            u64,
        )>(conn);

    if raw_reviews.is_err() {
        return Err(DbError::new(
            DbErrorKind::ReadFailed,
            "Could not query Reviews",
        ));
    }
    let raw_reviews = raw_reviews.unwrap();

    let reviews: Vec<TodoReview> = raw_reviews
        .into_iter()
        .map(|review| {
            let (firstname, lastname): (Option<String>, Option<String>) =
                if !db::workshops::is_anonymous(conn, review.8) {
                    (Some(review.4), Some(review.5))
                } else {
                    (None, None)
                };
            TodoReview {
                id: review.0,
                done: review.1,
                deadline: review.2,
                title: review.3,
                firstname,
                lastname,
                submission: review.6,
                workshop_name: review.7,
            }
        })
        .collect();

    // Get Workshops where no Submissions were placed yet
    // Also filter only current workshops
    // --------------------------------------------------
    let raw_submissions = workshops_t
        .left_outer_join(workshoplist_t.on(ws_id.eq(wsl_ws)))
        .left_outer_join(users_t.on(user_id.eq(wsl_user)))
        .filter(
            wsl_user
                .eq(student_id)
                .and(not(exists(
                    submissions_t.filter(sub_student.eq(student_id).and(sub_ws.eq(ws_id))),
                )))
                .and(ws_end.ge(now)),
        )
        .select((ws_id, ws_title))
        .get_results::<(u64, String)>(conn);

    if raw_submissions.is_err() {
        return Err(DbError::new(
            DbErrorKind::ReadFailed,
            "Could not query Submissions",
        ));
    }
    let raw_submissions = raw_submissions.unwrap();

    let submissions: Vec<TodoSubmission> = raw_submissions
        .into_iter()
        .map(|workshop| TodoSubmission {
            id: workshop.0,
            workshop_name: workshop.1,
        })
        .collect();

    /*

    // This is it
    select w.title, u.username
        from workshops w
        left outer join workshoplist wl on w.id=wl.workshop
        left outer join users u on wl.user=u.id
        where wl.user=5 and not exists(select * from submissions where student=5);

    select w.title, u.username
        from workshops w
        left outer join workshoplist wl on w.id=wl.workshop
        right outer join users u on wl.user=u.id
        where wl.user=5 and not exists(select * from submissions where student=5);

    select w.title, u.username
        from workshops w
        left outer join workshoplist wl on w.id=wl.workshop
        right outer join users u on wl.user=u.id;


    select w.title, u.username, count(s.student) as count
        from workshops w
        left outer join submissions s on w.id=s.workshop
        right outer join users u on s.student=u.id
        group by w.title, u.username;

    select w.title, u.username, count(s.student) as count
       from submissions s
       right outer join users u on u.id=s.student
       left outer join workshops w on w.id=s.workshop
       group by w.title, u.username;

    select w.title, u.username, (
        select count(*)
        from submissions
        where s.workshop=w.id and s.student=u.id
    ) as count
    from submissions s
    right outer join workshops w on s.workshop=w.id
    right outer join users u on s.student=u.id;


    select u.username, count(s.student) as count
       from submissions s
       right outer join users u on u.id=s.student
       group by u.username;

       select u.username, count(s.student) as count
       from submissions s
       right outer join users u on u.id=s.student
       group by u.username;

    */

    Ok(Todo {
        reviews,
        submissions,
    })
}
