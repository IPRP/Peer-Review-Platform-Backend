use crate::db;
use crate::db::reviews::FullReview;
use crate::models::{
    Criterion, Kind, NewSubmission, Role, SimpleAttachment, Submission, Submissionattachment,
    Submissioncriteria,
};
use crate::schema::criteria::dsl::workshop;
use crate::schema::criterion::dsl::{criterion as criterion_t, id as c_id};
use crate::schema::submissionattachments::dsl::{
    attachment as subatt_att, submission as subatt_sub, submissionattachments as subatt_t,
};
use crate::schema::submissioncriteria::dsl::{
    criterion as subcrit_crit, submission as subcrit_sub, submissioncriteria as subcrit_t,
};
use crate::schema::submissions::dsl::{
    error as sub_error, id as sub_id, locked as sub_locked, meanpoints as sub_meanpoints,
    reviewsdone as sub_reviews_done, student as sub_student, submissions as submissions_t,
};
use diesel::prelude::*;
use diesel::result::Error;
use std::ops::Add;

pub fn create<'a>(
    conn: &MysqlConnection,
    title: String,
    comment: String,
    attachments: Vec<u64>,
    date: chrono::NaiveDateTime,
    student_id: u64,
    workshop_id: u64,
) -> Result<Submission, ()> {
    // Check if student is part of the workshop
    if !db::workshops::student_in_workshop(conn, student_id, workshop_id) {
        return Err(());
    }

    let new_submission = NewSubmission {
        title,
        comment,
        student: student_id,
        workshop: workshop_id,
        date,
        locked: false,
        reviewsdone: false,
        error: false,
    };

    let submission = conn.transaction::<Submission, Error, _>(|| {
        // Insert submission
        diesel::insert_into(submissions_t)
            .values(&new_submission)
            .execute(conn);
        let submission: Submission = submissions_t.order(sub_id.desc()).first(conn).unwrap();

        // Relate attachments to submission
        let all_student_attachments = db::attachments::get_ids_by_user_id(conn, student_id);
        if all_student_attachments.is_err() {
            return Err(Error::RollbackTransaction);
        }
        let all_student_attachments = all_student_attachments.unwrap();
        let submission_attachments: Vec<Submissionattachment> = attachments
            .into_iter()
            .filter_map(|att_id| {
                if all_student_attachments.contains(&att_id) {
                    Some(Submissionattachment {
                        submission: submission.id,
                        attachment: att_id,
                    })
                } else {
                    None
                }
            })
            .collect();
        diesel::insert_into(subatt_t)
            .values(&submission_attachments)
            .execute(conn);

        // Relate criteria to submission
        let workshop_criteria = db::workshops::get_criteria(conn, workshop_id);
        if workshop_criteria.is_err() {
            return Err(Error::RollbackTransaction);
        }
        let workshop_criteria = workshop_criteria.unwrap();
        let submission_criteria: Vec<Submissioncriteria> = workshop_criteria
            .into_iter()
            .map(|criterion| Submissioncriteria {
                submission: submission.id,
                criterion,
            })
            .collect();
        diesel::insert_into(subcrit_t)
            .values(&submission_criteria)
            .execute(conn);

        // Assign reviews
        let assign = db::reviews::assign(conn, date, submission.id, student_id, workshop_id);
        if assign.is_err() {
            return Err(Error::RollbackTransaction);
        }

        Ok(submission)
    });

    match submission {
        Ok(submission) => Ok(submission),
        Err(_) => Err(()),
    }
}

pub fn is_owner(conn: &MysqlConnection, submission_id: u64, student_id: u64) -> bool {
    let exists: Result<Submission, diesel::result::Error> = submissions_t
        .filter(sub_id.eq(submission_id).and(sub_student.eq(student_id)))
        .first(conn);
    if exists.is_ok() {
        true
    } else {
        false
    }
}

// TODO reviews
#[derive(Serialize)]
pub struct OwnSubmission {
    pub title: String,
    pub comment: String,
    pub attachments: Vec<SimpleAttachment>,
    pub locked: bool,
    pub date: chrono::NaiveDateTime,
    #[serde(rename(serialize = "reviewsDone"))]
    pub reviews_done: bool,
    #[serde(rename(serialize = "noReviews"))]
    pub no_reviews: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<f64>,
    #[serde(rename(serialize = "maxPoints"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_points: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firstname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastname: Option<String>,
    pub reviews: Vec<FullReview>,
}

pub fn get_own_submission(
    conn: &MysqlConnection,
    submission_id: u64,
    user_id: u64,
) -> Result<OwnSubmission, ()> {
    let points_calculation = calculate_points(conn, submission_id);
    if points_calculation.is_err() {
        return Err(());
    }
    let attachments = db::attachments::get_by_submission_id(conn, submission_id);
    if attachments.is_err() {
        return Err(());
    }
    let attachments = attachments.unwrap();
    let submission: Result<Submission, _> = submissions_t
        .filter(sub_id.eq(submission_id).and(sub_student.eq(user_id)))
        .first(conn);
    if submission.is_err() {
        return Err(());
    }
    let submission = submission.unwrap();

    let no_reviews = if submission.meanpoints.is_none() {
        true
    } else {
        false
    };

    let reviews = if submission.reviewsdone {
        if let Ok(reviews) = db::reviews::get_full_reviews(conn, submission_id) {
            reviews
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Ok(OwnSubmission {
        title: submission.title,
        comment: submission.comment,
        attachments,
        locked: submission.locked,
        date: submission.date,
        reviews_done: submission.reviewsdone,
        no_reviews,
        points: submission.meanpoints,
        max_points: submission.maxpoint,
        firstname: None,
        lastname: None,
        reviews,
    })
}

#[derive(Serialize)]
pub struct OtherSubmission {
    pub title: String,
    pub comment: String,
    pub attachments: Vec<SimpleAttachment>,
    pub criteria: Vec<Criterion>,
}

pub fn get_student_submission(
    conn: &MysqlConnection,
    submission_id: u64,
    _user_id: u64,
) -> Result<OtherSubmission, ()> {
    let points_calculation = calculate_points(conn, submission_id);
    if points_calculation.is_err() {
        return Err(());
    }
    let attachments =
        db::attachments::get_by_submission_id_and_lock_submission(conn, submission_id);
    if attachments.is_err() {
        return Err(());
    }
    let attachments = attachments.unwrap();

    let submission: Result<Submission, _> =
        submissions_t.filter(sub_id.eq(submission_id)).first(conn);
    if submission.is_err() {
        return Err(());
    }
    let submission = submission.unwrap();

    let submission_criteria: Result<Vec<u64>, _> = subcrit_t
        .filter(subcrit_sub.eq(submission_id))
        .select(subcrit_crit)
        .get_results(conn);
    if submission_criteria.is_err() {
        return Err(());
    }
    let submission_criteria = submission_criteria.unwrap();

    let submission_criteria: Result<Vec<Criterion>, _> = criterion_t
        .filter(c_id.eq_any(submission_criteria))
        .get_results(conn);
    if submission_criteria.is_err() {
        return Err(());
    }
    let submission_criteria = submission_criteria.unwrap();

    Ok(OtherSubmission {
        title: submission.title,
        comment: submission.comment,
        attachments,
        criteria: submission_criteria,
    })
}

fn calculate_points(conn: &MysqlConnection, submission_id: u64) -> Result<(), ()> {
    let submission = submissions_t
        .filter(
            sub_id.eq(submission_id).and(
                sub_reviews_done
                    .eq(true)
                    .and(sub_meanpoints.is_null().and(sub_error.eq(false))),
            ),
        )
        .first(conn);

    if submission.is_err() {
        // Submission points are already calculated or not finished yet
        return Ok(());
    }
    let mut submission: Submission = submission.unwrap();
    // Get all reviews without errors
    let reviews = db::reviews::get_simple_review_points(conn, submission_id);
    if reviews.is_err() {
        return Err(());
    }
    let reviews = reviews.unwrap();
    // Perform updates
    let res = conn.transaction::<_, _, _>(|| {
        // If there are no reviews, a submissions cannot be graded
        if reviews.len() == 0 {
            // Save error state
            submission.error = true;
            let update = diesel::update(submissions_t).set(&submission).execute(conn);
            if update.is_err() {
                return Err(Error::RollbackTransaction);
            }
        } else {
            // Calculate mean points and max points (based on criterion and weights)
            // Max points
            let point_range = 10.0;
            let mut max_points = 0.0;
            for points in reviews.first() {
                for point in points {
                    max_points += point_range * point.weight;
                }
            }
            // Mean points
            let mut mean_points = 0.0;
            for points in reviews.iter() {
                let mut review_mean_points = 0.0;
                for point in points {
                    let weighted_points = match point.kind {
                        Kind::Point => (point.points % point_range) * point.weight,
                        Kind::Grade => match point.points {
                            1.0 => point_range,
                            2.0 => point_range * 0.8,
                            3.0 => point_range * 0.6,
                            4.0 => point_range * 0.5,
                            _ => 0.0,
                        },
                        Kind::Percentage => ((point.points % 100.0) / point_range) * point.weight,
                        Kind::Truefalse => match point.points {
                            1.0 => point_range * point.weight,
                            _ => 0.0,
                        },
                    };
                    review_mean_points += weighted_points;
                }
                mean_points += review_mean_points;
            }
            mean_points /= reviews.len() as f64;
            // Update submission
            submission.maxpoint = Some(max_points);
            submission.meanpoints = Some(mean_points);
            let update = diesel::update(submissions_t).set(&submission).execute(conn);
            if update.is_err() {
                return Err(Error::RollbackTransaction);
            }
        }
        Ok(())
    });
    match res {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

pub fn get_criteria(conn: &MysqlConnection, submission_id: u64) -> Result<Vec<Criterion>, ()> {
    let submission_criteria: Result<Vec<u64>, _> = subcrit_t
        .filter(subcrit_sub.eq(submission_id))
        .select(subcrit_crit)
        .get_results(conn);
    if submission_criteria.is_err() {
        return Err(());
    }
    let submission_criteria = submission_criteria.unwrap();

    let submission_criteria: Result<Vec<Criterion>, _> = criterion_t
        .filter(c_id.eq_any(submission_criteria))
        .get_results(conn);
    if submission_criteria.is_err() {
        return Err(());
    }
    Ok(submission_criteria.unwrap())
}

pub fn lock(conn: &MysqlConnection, submission_id: u64) -> Result<(), ()> {
    let update = conn.transaction::<_, _, _>(|| {
        let submission = submissions_t.filter(sub_id.eq(submission_id));
        let update = diesel::update(submission)
            .set(sub_locked.eq(true))
            .execute(conn);
        if update.is_err() {
            return Err(Error::RollbackTransaction);
        }
        Ok(())
    });
    match update {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

pub fn get_by_id(conn: &MysqlConnection, submission_id: u64) -> Result<Submission, Error> {
    submissions_t.filter(sub_id.eq(submission_id)).first(conn)
}
