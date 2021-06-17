use crate::db;
use crate::models::{Kind, NewReview, Review, ReviewPoints, Role};
use crate::routes::submissions::UpdateReview;
use crate::schema::reviewpoints::dsl::reviewpoints as reviewpoints_t;
use crate::schema::reviews::dsl::{
    id as reviews_id, reviewer, reviews as reviews_t, submission as reviews_sub,
};
use crate::schema::workshoplist::dsl::{
    role as wsl_role, user as wsl_user, workshop as wsl_ws, workshoplist as workshoplist_t,
};
use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone, Utc};
use diesel::connection::SimpleConnection;
use diesel::dsl::count;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_types::BigInt;
use std::convert::TryInto;
use std::ops::Add;

pub fn assign(
    conn: &MysqlConnection,
    date: chrono::NaiveDateTime,
    submission_id: u64,
    submission_student_id: u64,
    workshop_id: u64,
) -> Result<(), ()> {
    // Calculate deadline
    // TODO set correct deadline
    //let deadline = date + Duration::days(7);
    let deadline = date + Duration::minutes(1);

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
    // TODO error handling, calculating mean points
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

pub fn update(
    conn: &MysqlConnection,
    update_review: UpdateReview,
    review_id: u64,
    user_id: u64,
) -> bool {
    let res = conn.transaction::<_, _, _>(|| {
        let review: Result<Review, _> = reviews_t
            .filter(reviews_id.eq(review_id).and(reviewer.eq(user_id)))
            .first(conn);
        if review.is_err() {
            return Err(Error::RollbackTransaction);
        }
        let mut review = review.unwrap();
        review.feedback = update_review.feedback;
        review.done = true;
        diesel::update(reviews_t).set(&review).execute(conn);

        // TODO check if all point criteria were given
        // Submission has 2 criteria, review needs to have two filled out criteria

        let review_points: Vec<ReviewPoints> = update_review
            .points
            .into_iter()
            .map(|update_points| ReviewPoints {
                review: review_id,
                criterion: update_points.id,
                points: update_points.points,
            })
            .collect();
        diesel::insert_into(reviewpoints_t)
            .values(&review_points)
            .execute(conn);

        Ok(())
    });

    match res {
        Ok(_) => true,
        Err(_) => false,
    }
}

/**
#[derive(Serialize)]
pub struct FullReview {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firstname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastname: Option<String>,
    pub feedback: String,
    pub points: Vec<ReviewPoints>,
}

#[derive(Debug, Serialize)]
pub struct ReviewPoints {
    title: String,
    content: String,
    weight: f64,
    #[serde(rename = "type")]
    kind: Kind,
    #[serde(skip_serializing_if = "Option::is_none")]
    points: Option<f64>,
}*/

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
