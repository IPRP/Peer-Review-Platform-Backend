use crate::db;
use crate::models::Role;
use crate::schema::reviews::dsl::{reviewer, reviews as reviews_t};
use crate::schema::workshoplist::dsl::{
    role as wsl_role, user as wsl_user, workshop as wsl_ws, workshoplist as workshoplist_t,
};
use chrono::Duration;
use diesel::dsl::count;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_types::BigInt;
use std::ops::Add;

pub fn assign(
    conn: &MysqlConnection,
    date: chrono::NaiveDateTime,
    submission_id: u64,
    submission_student_id: u64,
    workshop_id: u64,
) -> Result<(), ()> {
    // Calculate deadline
    let deadline = date + Duration::days(7);
    // Get (at max) 3 users who have the least count of reviews for this particular workshop
    /*
    Based on: https://stackoverflow.com/a/2838527/12347616
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
    let reviews = reviews.unwrap();
    println!("Reviews: {:?}", reviews);

    // Assign reviews to them

    // Create events that close reviews
    Ok(())
}
