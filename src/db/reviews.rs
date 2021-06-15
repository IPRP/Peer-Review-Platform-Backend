use crate::db;
use crate::models::{NewReview, Review, Role};
use crate::schema::reviews::dsl::{id as reviews_id, reviewer, reviews as reviews_t};
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
    let deadline = date + Duration::minutes(2);

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
    // https://stackoverflow.com/a/8763381/12347616
    for review in reviews {
        // See: https://docs.rs/chrono/0.4.0/chrono/naive/struct.NaiveDateTime.html#example-14
        // And: https://dev.mysql.com/doc/refman/8.0/en/create-event.html
        let id = review.id;
        // Convert local time to UTC
        // Why? MySQL Events use internally UTC time to determine when to trigger them...
        // See: https://dba.stackexchange.com/a/255569
        // And: https://stackoverflow.com/a/65830964/12347616
        let timestamp = Local.from_local_datetime(&review.deadline).unwrap();
        let timestamp = date_time.with_timezone(&Utc);
        let timestamp = date_time.format("%Y-%m-%d %H:%M:%S").to_string();
        /*let timestamp = (DateTime::<Utc>::from_utc(review.deadline, Utc))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();*/
        println!("{}", timestamp);
        let res = conn.batch_execute(&*format!(
            r#"
        DROP EVENT IF EXISTS close_review_{id};
        CREATE EVENT close_review_{id}
        ON SCHEDULE AT '{timestamp}'
        DO
          UPDATE reviews
          SET done = 1, locked = 1
          WHERE id = {id};
    "#,
            id = id,
            timestamp = timestamp
        ));
        if res.is_err() {
            println!("{}", res.err().unwrap());
            return Err(());
        }
    }
    Ok(())
}
