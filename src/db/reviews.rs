//! CRUD operations for reviews.

use crate::db;
use crate::db::ReviewTimespan;
use crate::models::{Kind, NewReview, Review, ReviewPoints, Role};
use crate::routes::submissions::UpdateReview;
use crate::schema::criterion::dsl::{
    content as c_content, criterion as criterion_t, id as c_id, kind as c_kind, title as c_title,
    weight as c_weight,
};
use crate::schema::reviewpoints::dsl::{
    criterion as rp_criterion, points as rp_points, review as rp_review,
    reviewpoints as reviewpoints_t,
};
use crate::schema::reviews::dsl::{
    deadline as reviews_deadline, done as reviews_done, error as reviews_error, id as reviews_id,
    reviewer, reviews as reviews_t, submission as reviews_sub,
};
use crate::schema::submissions::dsl::{
    id as sub_id, student as sub_student, submissions as submissions_t, title as sub_title,
    workshop as sub_ws,
};
use crate::schema::users::dsl::{
    firstname as u_firstname, id as u_id, lastname as u_lastname, users as users_t,
};
use crate::schema::workshoplist::dsl::{
    role as wsl_role, user as wsl_user, workshop as wsl_ws, workshoplist as workshoplist_t,
};
use crate::schema::workshops::dsl::{id as ws_id, title as ws_title, workshops as workshops_t};
use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone, Utc};
use diesel::connection::SimpleConnection;
use diesel::dsl::count;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_types::BigInt;
use std::convert::TryInto;
use std::ops::Add;

/// Assign reviews from a given submission.
pub fn assign(
    conn: &MysqlConnection,
    review_timespan: &ReviewTimespan,
    date: chrono::NaiveDateTime,
    submission_id: u64,
    submission_student_id: u64,
    workshop_id: u64,
) -> Result<(), ()> {
    // Calculate deadline
    let deadline = review_timespan.deadline(&date);

    // Get (at max) 3 users who have the least count of reviews for this particular workshop
    /* Based on: https://stackoverflow.com/a/2838527/12347616
           select user, count(reviewer)
           from workshoplist w
           left outer join reviews r on w.user=r.reviewer
           where w.workshop=1 and w.role="student"
           Group by user
           order by count(reviewer) DESC, user;
    */
    // Nullable eq: https://docs.diesel.rs/diesel/expression_methods/trait.NullableExpressionMethods.html
    // Problems with count: https://github.com/diesel-rs/diesel/issues/1781
    let count_reviewer = diesel::dsl::sql::<diesel::sql_types::Unsigned<BigInt>>("count(reviewer)");
    let reviews = workshoplist_t
        .left_outer_join(reviews_t.on(reviewer.nullable().eq(wsl_user.nullable())))
        .filter(
            wsl_ws.eq(workshop_id).and(
                wsl_role
                    .eq(Role::Student)
                    .and(wsl_user.ne(submission_student_id)),
            ),
        )
        .group_by(wsl_user)
        .order((count_reviewer.clone().desc(), wsl_user))
        .select((wsl_user, count_reviewer))
        .limit(3)
        .get_results::<(u64, u64)>(conn);
    if reviews.is_err() {
        return Err(());
    }
    let reviews: Vec<(u64, u64)> = reviews.unwrap();
    let review_count = reviews.len();
    //println!("Reviews: {:?}", reviews);

    // Assign reviews to them
    let reviews: Vec<NewReview> = reviews
        .into_iter()
        .map(|review| NewReview {
            feedback: "".to_string(),
            reviewer: Some(review.0),
            submission: submission_id,
            workshop: workshop_id,
            deadline,
            done: false,
            locked: false,
            error: false,
        })
        .collect();
    diesel::insert_into(reviews_t)
        .values(&reviews)
        .execute(conn);
    let reviews = reviews_t
        .order(reviews_id.desc())
        .limit(review_count.try_into().unwrap())
        .get_results::<Review>(conn);
    if reviews.is_err() {
        return Err(());
    }
    let reviews: Vec<Review> = reviews.unwrap();

    // Create events that close reviews
    // See: https://docs.rs/chrono/0.4.0/chrono/naive/struct.NaiveDateTime.html#example-14
    // And: https://dev.mysql.com/doc/refman/8.0/en/create-event.html
    // And: https://stackoverflow.com/a/8763381/12347616

    // Convert local time to UTC
    // Why? MySQL Events use internally UTC time to determine when to trigger them...
    // See: https://dba.stackexchange.com/a/255569
    // And: https://stackoverflow.com/a/65830964/12347616
    let deadline = Local.from_local_datetime(&deadline).unwrap();
    let deadline = deadline.with_timezone(&Utc);
    let timestamp = deadline.format("%Y-%m-%d %H:%M:%S").to_string();
    println!("{}", timestamp);

    for review in reviews {
        let res = conn.batch_execute(&*format!(
            r#"
        DROP EVENT IF EXISTS close_review_{id};
        CREATE EVENT close_review_{id}
        ON SCHEDULE AT '{timestamp}'
        DO
          UPDATE reviews
          SET done = 1, locked = 1, 
          error = CASE
            WHEN exists(select * from reviewpoints where review={id}) THEN 0
            ELSE 1
          END
          WHERE id = {id};
    "#,
            id = review.id,
            timestamp = timestamp
        ));
        if res.is_err() {
            println!("{}", res.err().unwrap());
            return Err(());
        }
    }

    // Create event that closes submission
    let deadline = deadline + Duration::seconds(5);
    let timestamp = deadline.format("%Y-%m-%d %H:%M:%S").to_string();
    let res = conn.batch_execute(&*format!(
        r#"
        DROP EVENT IF EXISTS close_submission_{id};
        CREATE EVENT close_submission_{id}
        ON SCHEDULE AT '{timestamp}'
        DO
          UPDATE submissions
          SET reviewsdone = 1, locked = 1, 
          error = CASE
            WHEN exists(select * from reviews where submission={id} and error=0) THEN 0
            ELSE 1
          END
          WHERE id = {id};
    "#,
        id = submission_id,
        timestamp = timestamp
    ));
    if res.is_err() {
        println!("{}", res.err().unwrap());
        return Err(());
    }
    /*
    select r.id, r.done, r.deadline, s.id, s.student, s.workshop
        from reviews r
        inner join submissions s on r.submission=s.id
        where s.student=4;

    select r.id, r.done, r.deadline, s.id, s.student, w.title
        from reviews r
        inner join submissions s on r.submission=s.id
        inner join workshops w on s.workshop=w.id
        where s.student=4;

    select r.id, r.done, r.deadline, s.id, s.student, w.title
        from reviews r
        inner join submissions s on r.submission=s.id
        inner join workshops w on s.workshop=w.id
        where s.student=4;

    select r.id, r.done, r.deadline, s.id, u.firstname, u.lastname, w.title
        from reviews r
        inner join submissions s on r.submission=s.id
        inner join workshops w on s.workshop=w.id
        inner join users u on s.student=u.id
        where s.student=4;
     */

    Ok(())
}

/// Update review.
/// Can be performed multiple times until review is locked on deadline.
pub fn update(
    conn: &MysqlConnection,
    update_review: UpdateReview,
    review_id: u64,
    user_id: u64,
) -> bool {
    // Get review
    let review: Result<Review, _> = reviews_t
        .filter(reviews_id.eq(review_id).and(reviewer.eq(user_id)))
        .first(conn);
    if review.is_err() {
        // No matching review
        return false;
    }
    let mut review = review.unwrap();
    if Local::now().naive_local() > review.deadline {
        // Update past deadline
        return false;
    }
    // Update review
    let res = conn.transaction::<_, _, _>(|| {
        // Check if all point criteria were given for update
        let criteria = db::submissions::get_criteria(conn, review.submission);
        if criteria.is_err() {
            return Err(Error::RollbackTransaction);
        }
        let criteria = criteria.unwrap();
        let update_ids: Vec<u64> = update_review
            .points
            .iter()
            .map(|update_points| update_points.id)
            .collect();
        for criterion in criteria {
            if !update_ids.contains(&criterion.id) {
                return Err(Error::RollbackTransaction);
            }
        }

        // Update review
        review.feedback = update_review.feedback;
        review.done = true;
        diesel::update(reviews_t.filter(reviews_id.eq(review.id)))
            .set(&review)
            .execute(conn);

        // Update review points
        // First `update_review` needs to be changed into a insertable form
        let review_points: Vec<ReviewPoints> = update_review
            .points
            .into_iter()
            .map(|update_points| ReviewPoints {
                review: review_id,
                criterion: update_points.id,
                points: update_points.points,
            })
            .collect();

        // Drop already given review points
        let delete = diesel::delete(reviewpoints_t.filter(rp_review.eq(review_id))).execute(conn);
        if delete.is_err() {
            return Err(Error::RollbackTransaction);
        }

        // Insert new review points
        let insert = diesel::insert_into(reviewpoints_t)
            .values(&review_points)
            .execute(conn);
        if insert.is_err() {
            return Err(Error::RollbackTransaction);
        }

        Ok(())
    });

    match res {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Simplified representation of review points.
#[derive(Serialize)]
pub struct SimpleReviewPoints {
    pub weight: f64,
    pub kind: Kind,
    pub points: f64,
}

/// Get simplified review points from a submission.
pub fn get_simple_review_points(
    conn: &MysqlConnection,
    submission_id: u64,
) -> Result<Vec<Vec<SimpleReviewPoints>>, ()> {
    let reviews = reviews_t
        .filter(reviews_sub.eq(submission_id).and(reviews_error.eq(false)))
        .select(reviews_id)
        .get_results::<u64>(conn);
    if reviews.is_err() {
        return Err(());
    }
    let reviews: Vec<u64> = reviews.unwrap();

    let mut simple_reviews: Vec<Vec<SimpleReviewPoints>> = Vec::new();
    for review in reviews {
        let points = criterion_t
            .inner_join(reviewpoints_t.on(c_id.eq(rp_criterion)))
            .filter(rp_review.eq(review))
            .select((c_weight, c_kind, rp_points))
            .get_results::<(f64, Kind, Option<f64>)>(conn);
        if points.is_err() {
            return Err(());
        }
        let points: Vec<(f64, Kind, Option<f64>)> = points.unwrap();
        let points: Vec<SimpleReviewPoints> = points
            .into_iter()
            .map(|point| SimpleReviewPoints {
                weight: point.0,
                kind: point.1,
                points: point.2.unwrap(),
            })
            .collect();
        simple_reviews.push(points);
    }
    Ok(simple_reviews)
}

/// Detailed representation of a review.
#[derive(Serialize)]
pub struct FullReview {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firstname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastname: Option<String>,
    pub feedback: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "notSubmitted")]
    pub not_submitted: Option<bool>,
    pub points: Vec<FullReviewPoints>,
}

/// Detailed representation of review points.
#[derive(Debug, Serialize)]
pub struct FullReviewPoints {
    #[serde(rename = "id")]
    pub criterion_id: u64,
    pub title: String,
    pub content: String,
    pub weight: f64,
    #[serde(rename = "type")]
    pub kind: Kind,
    pub points: f64,
}

// Get all detailed reviews from a submission.
fn get_full_reviews_internal(
    conn: &MysqlConnection,
    submission_id: u64,
    with_names: bool,
) -> Result<Vec<FullReview>, ()> {
    let reviews = reviews_t
        .filter(reviews_sub.eq(submission_id).and(reviews_error.eq(false)))
        .get_results::<Review>(conn);
    if reviews.is_err() {
        return Err(());
    }
    let reviews: Vec<Review> = reviews.unwrap();

    let mut full_reviews: Vec<FullReview> = Vec::new();
    for review in reviews.iter() {
        let points = criterion_t
            .inner_join(reviewpoints_t.on(c_id.eq(rp_criterion)))
            .filter(rp_review.eq(review.id))
            .select((c_id, c_title, c_content, c_weight, c_kind, rp_points))
            .get_results::<(u64, String, String, f64, Kind, Option<f64>)>(conn);
        if points.is_err() {
            return Err(());
        }
        let points: Vec<(u64, String, String, f64, Kind, Option<f64>)> = points.unwrap();
        let points: Vec<FullReviewPoints> = points
            .into_iter()
            .map(|point| FullReviewPoints {
                criterion_id: point.0,
                title: point.1,
                content: point.2,
                weight: point.3,
                kind: point.4,
                points: point.5.unwrap(),
            })
            .collect();
        let (firstname, lastname) = if with_names && review.reviewer.is_some() {
            let user = db::users::get_by_id(conn, review.reviewer.unwrap());
            if user.is_ok() {
                let user = user.unwrap();
                (Some(user.firstname), Some(user.lastname))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };
        full_reviews.push(FullReview {
            id: review.id,
            firstname,
            lastname,
            feedback: review.feedback.clone(),
            not_submitted: None,
            points,
        });
    }
    Ok(full_reviews)
}

/// Get all detailed reviews from a submission.
/// If workshop is anonymous no names will be returned.
pub fn get_full_reviews(conn: &MysqlConnection, submission_id: u64) -> Result<Vec<FullReview>, ()> {
    let workshop = db::workshops::get_by_submission_id(conn, submission_id);
    let with_names = if workshop.is_ok() {
        !workshop.unwrap().anonymous
    } else {
        false
    };
    get_full_reviews_internal(conn, submission_id, with_names)
}

/// Get all detailed reviews from a submission with names.
pub fn get_full_reviews_with_names(
    conn: &MysqlConnection,
    submission_id: u64,
) -> Result<Vec<FullReview>, ()> {
    get_full_reviews_internal(conn, submission_id, true)
}

// Get detailed review.
fn get_full_review_internal(
    conn: &MysqlConnection,
    review_id: u64,
    with_names: bool,
) -> Result<FullReview, ()> {
    let review = reviews_t
        .filter(reviews_id.eq(review_id))
        .first::<Review>(conn);
    if review.is_err() {
        return Err(());
    }
    let review: Review = review.unwrap();

    let points = if review.done && !review.error {
        let points = criterion_t
            .inner_join(reviewpoints_t.on(c_id.eq(rp_criterion)))
            .filter(rp_review.eq(review.id))
            .select((c_id, c_title, c_content, c_weight, c_kind, rp_points))
            .get_results::<(u64, String, String, f64, Kind, Option<f64>)>(conn);
        if points.is_err() {
            Vec::new()
        } else {
            let points: Vec<(u64, String, String, f64, Kind, Option<f64>)> = points.unwrap();
            points
                .into_iter()
                .map(|point| FullReviewPoints {
                    criterion_id: point.0,
                    title: point.1,
                    content: point.2,
                    weight: point.3,
                    kind: point.4,
                    points: point.5.unwrap(),
                })
                .collect()
        }
    } else {
        Vec::new()
    };

    let (firstname, lastname) = if with_names && review.reviewer.is_some() {
        let user = db::users::get_by_id(conn, review.reviewer.unwrap());
        if user.is_ok() {
            let user = user.unwrap();
            (Some(user.firstname), Some(user.lastname))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    Ok(FullReview {
        id: review.id,
        firstname,
        lastname,
        feedback: review.feedback.clone(),
        not_submitted: Some(review.error),
        points,
    })
}

/// Get detailed review.
/// If workshop is anonymous no names will be returned.
pub fn get_full_review(conn: &MysqlConnection, review_id: u64) -> Result<FullReview, ()> {
    let workshop = db::workshops::get_by_submission_id(conn, review_id);
    let with_names = if workshop.is_ok() {
        !workshop.unwrap().anonymous
    } else {
        false
    };
    get_full_review_internal(conn, review_id, with_names)
}

/// Get detailed review with names.
pub fn get_full_review_with_names(
    conn: &MysqlConnection,
    review_id: u64,
) -> Result<FullReview, ()> {
    get_full_review_internal(conn, review_id, true)
}

/// Check if student is review for a given submission.
pub fn is_reviewer(conn: &MysqlConnection, submission_id: u64, student_id: u64) -> bool {
    let exists: Result<Review, diesel::result::Error> = reviews_t
        .filter(reviewer.eq(student_id).and(reviews_sub.eq(submission_id)))
        .first(conn);
    if exists.is_ok() {
        true
    } else {
        false
    }
}

/// Check if student is reviewer of given review.
pub fn is_owner(conn: &MysqlConnection, review_id: u64, student_id: u64) -> bool {
    let exists: Result<Review, diesel::result::Error> = reviews_t
        .filter(reviews_id.eq(review_id).and(reviewer.eq(student_id)))
        .first(conn);
    if exists.is_ok() {
        true
    } else {
        false
    }
}

/// Check if student is submission owner through a review of this submission.
pub fn is_submission_owner(conn: &MysqlConnection, review_id: u64, student_id: u64) -> bool {
    let review: Result<Review, diesel::result::Error> =
        reviews_t.filter(reviews_id.eq(review_id)).first(conn);
    if review.is_ok() {
        let review = review.unwrap();
        let submission = db::submissions::get_by_id(conn, review.submission);
        if submission.is_ok() {
            let submission = submission.unwrap();
            if let Some(student) = submission.student {
                student == student_id
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// Get review by review id.
pub fn get_by_id(conn: &MysqlConnection, review_id: u64) -> Result<Review, Error> {
    reviews_t.filter(reviews_id.eq(review_id)).first(conn)
}

/// Workshop representation of a review.
#[derive(Serialize)]
pub struct WorkshopReview {
    pub id: u64,
    pub done: bool,
    pub deadline: chrono::NaiveDateTime,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firstname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastname: Option<String>,
}

/// Get all reviews from a student in one workshop.
pub fn get_student_workshop_reviews(
    conn: &MysqlConnection,
    workshop_id: u64,
    student_id: u64,
) -> Result<Vec<WorkshopReview>, ()> {
    let raw_reviews = reviews_t
        .inner_join(submissions_t.on(sub_id.eq(reviews_sub)))
        .inner_join(workshops_t.on(ws_id.eq(sub_ws)))
        .inner_join(users_t.on(u_id.nullable().eq(sub_student.nullable())))
        .filter(reviewer.eq(student_id).and(ws_id.eq(workshop_id)))
        .select((
            reviews_id,
            reviews_done,
            reviews_deadline,
            sub_title,
            u_firstname,
            u_lastname,
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
        )>(conn);

    if raw_reviews.is_err() {
        return Err(());
    }
    let raw_reviews = raw_reviews.unwrap();

    let reviews: Vec<WorkshopReview> = raw_reviews
        .into_iter()
        .map(|review| {
            let (firstname, lastname): (Option<String>, Option<String>) =
                if !db::workshops::is_anonymous(conn, review.6) {
                    (Some(review.4), Some(review.5))
                } else {
                    (None, None)
                };
            WorkshopReview {
                id: review.0,
                done: review.1,
                deadline: review.2,
                title: review.3,
                firstname,
                lastname,
            }
        })
        .collect();
    Ok(reviews)
}
