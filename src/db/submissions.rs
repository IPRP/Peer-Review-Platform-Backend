//! CRUD operations for submissions.

use crate::db;
use crate::db::models::*;
use crate::db::ReviewTimespan;
use chrono::{Duration, Local};
use std::ops::Add;

use crate::db::error::{DbError, DbErrorKind};
use crate::schema::criterion::dsl::{criterion as criterion_t, id as c_id};
use crate::schema::submissionattachments::dsl::submissionattachments as subatt_t;
use crate::schema::submissioncriteria::dsl::{
    criterion as subcrit_crit, submission as subcrit_sub, submissioncriteria as subcrit_t,
};
use crate::schema::submissions::dsl::{
    deadline as sub_deadline, error as sub_error, id as sub_id, locked as sub_locked,
    meanpoints as sub_meanpoints, reviewsdone as sub_reviews_done, student as sub_student,
    submissions as submissions_t, workshop as sub_workshop,
};
use diesel::prelude::*;
use diesel::result::Error;

/// Create a new submission for a workshop.
pub fn create<'a>(
    conn: &MysqlConnection,
    title: String,
    comment: String,
    attachments: Vec<u64>,
    date: chrono::NaiveDateTime,
    student_id: u64,
    workshop_id: u64,
) -> Result<Submission, DbError> {
    // Check if student is part of the workshop
    if !db::workshops::student_in_workshop(conn, student_id, workshop_id) {
        return Err(DbError::new(
            DbErrorKind::NotFound,
            format!("Student {} not in Workshop {}", student_id, workshop_id),
        ));
    }

    // Calculate deadline with review timespan from workshop
    let review_timespan = crate::db::workshops::get_review_timespan(conn, workshop_id);
    if let Err(err) = review_timespan {
        return Err(err);
    }
    let review_timespan = review_timespan.unwrap();
    let deadline = date.add(review_timespan);

    let new_submission = NewSubmission {
        title,
        comment,
        student: student_id,
        workshop: workshop_id,
        date,
        deadline,
        locked: false,
        reviewsdone: false,
        error: false,
    };

    let mut t_error: Result<(), DbError> = Ok(());
    let submission = conn.transaction::<Submission, Error, _>(|| {
        // Insert submission
        let submission_insert = diesel::insert_into(submissions_t)
            .values(&new_submission)
            .execute(conn);
        if submission_insert.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::CreateFailed, "Submission Insert failed"),
            );
        }
        let submission: Submission = submissions_t.order(sub_id.desc()).first(conn).unwrap();

        // Relate attachments to submission
        let all_student_attachments = db::attachments::get_ids_by_user_id(conn, student_id);
        if all_student_attachments.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::ReadFailed, "Student attachments not found"),
            );
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
        let attachment_insert = diesel::insert_into(subatt_t)
            .values(&submission_attachments)
            .execute(conn);
        if attachment_insert.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::CreateFailed, "Attachment Insert failed"),
            );
        }

        // Relate criteria to submission
        let workshop_criteria = db::workshops::get_criteria(conn, workshop_id);
        if workshop_criteria.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::ReadFailed, "Workshop Criteria not found"),
            );
        }
        let workshop_criteria = workshop_criteria.unwrap();
        let submission_criteria: Vec<Submissioncriteria> = workshop_criteria
            .into_iter()
            .map(|criterion| Submissioncriteria {
                submission: submission.id,
                criterion,
            })
            .collect();
        let criteria_insert = diesel::insert_into(subcrit_t)
            .values(&submission_criteria)
            .execute(conn);
        if criteria_insert.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::CreateFailed, "Criteria Insert failed"),
            );
        }

        // Assign reviews
        let assign = db::reviews::assign(conn, submission.id, student_id, workshop_id, deadline);
        if assign.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                assign.err().unwrap_or(DbError::new(
                    DbErrorKind::TransactionFailed,
                    "Review Assignment failed",
                )),
            );
        }

        Ok(submission)
    });

    match submission {
        Ok(submission) => Ok(submission),
        Err(_) => Err(t_error.err().unwrap_or(DbError::new(
            DbErrorKind::TransactionFailed,
            "Unknown error",
        ))),
    }
}

/// Check if student is owner of submission.
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

// Get detailed submission from submission id.
fn get_full_submission(
    conn: &MysqlConnection,
    submission_id: u64,
    is_teacher: bool,
) -> Result<OwnSubmission, DbError> {
    let points_calculation = calculate_points(conn, submission_id);
    if let Err(err) = points_calculation {
        return Err(err);
    }
    let attachments = db::attachments::get_by_submission_id(conn, submission_id);
    if attachments.is_err() {
        return Err(DbError::new(
            DbErrorKind::NotFound,
            format!("No Attachments for Submission {} found", submission_id),
        ));
    }
    let attachments = attachments.unwrap();
    let submission: Result<Submission, _> =
        submissions_t.filter(sub_id.eq(submission_id)).first(conn);
    if submission.is_err() {
        return Err(DbError::new(
            DbErrorKind::NotFound,
            format!("Submission {} not found", submission_id),
        ));
    }
    let submission = submission.unwrap();

    let (firstname, lastname) = if let Some(student) = submission.student {
        if let Ok(student) = db::users::get_by_id(conn, student) {
            (Some(student.firstname), Some(student.lastname))
        } else {
            (Some(String::from("Not Found")), Some(String::from("")))
        }
    } else {
        (Some(String::from("Not Found")), Some(String::from("")))
    };

    let no_reviews = if submission.meanpoints.is_none() {
        true
    } else {
        false
    };

    let reviews = if submission.reviewsdone {
        if is_teacher {
            if let Ok(reviews) = db::reviews::get_full_reviews_with_names(conn, submission_id) {
                reviews
            } else {
                Vec::new()
            }
        } else {
            if let Ok(reviews) = db::reviews::get_full_reviews(conn, submission_id) {
                reviews
            } else {
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    let missing_reviews = if submission.reviewsdone {
        if is_teacher {
            if let Ok(missing_reviews) = db::reviews::get_missing_reviews(conn, submission_id) {
                Some(missing_reviews)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
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
        firstname,
        lastname,
        reviews,
        missing_reviews,
    })
}

/// Get detailed submission from submission id.
pub fn get_own_submission(
    conn: &MysqlConnection,
    submission_id: u64,
) -> Result<OwnSubmission, DbError> {
    get_full_submission(conn, submission_id, false)
}

/// Get detailed submission from submission id.
pub fn get_teacher_submission(
    conn: &MysqlConnection,
    submission_id: u64,
) -> Result<OwnSubmission, DbError> {
    get_full_submission(conn, submission_id, true)
}

/// Get simplified submission from submission id.
/// Also locks submission so that submission owner can no longer update it.
pub fn get_student_submission(
    conn: &MysqlConnection,
    submission_id: u64,
    _user_id: u64,
) -> Result<OtherSubmission, DbError> {
    let points_calculation = calculate_points(conn, submission_id);
    if let Err(err) = points_calculation {
        return Err(err);
    }
    let attachments =
        db::attachments::get_by_submission_id_and_lock_submission(conn, submission_id);
    if let Err(err) = attachments {
        return Err(err);
    }
    let attachments = attachments.ok().unwrap();

    let submission: Result<Submission, _> =
        submissions_t.filter(sub_id.eq(submission_id)).first(conn);
    if submission.is_err() {
        return Err(DbError::new(
            DbErrorKind::ReadFailed,
            format!("Submission {} not found", submission_id),
        ));
    }
    let submission = submission.unwrap();

    let submission_criteria: Result<Vec<u64>, _> = subcrit_t
        .filter(subcrit_sub.eq(submission_id))
        .select(subcrit_crit)
        .get_results(conn);
    if submission_criteria.is_err() {
        return Err(DbError::new(
            DbErrorKind::ReadFailed,
            format!("Criteria for Submission {} not found", submission_id),
        ));
    }
    let submission_criteria = submission_criteria.unwrap();

    let submission_criteria: Result<Vec<Criterion>, _> = criterion_t
        .filter(c_id.eq_any(submission_criteria))
        .get_results(conn);
    if submission_criteria.is_err() {
        return Err(DbError::new(
            DbErrorKind::ReadFailed,
            format!("Criteria Data for Submission {} not found", submission_id),
        ));
    }
    let submission_criteria = submission_criteria.unwrap();

    Ok(OtherSubmission {
        title: submission.title,
        comment: submission.comment,
        attachments,
        criteria: submission_criteria,
    })
}

// Get all workshop submissions.
fn get_workshop_submissions_internal(
    conn: &MysqlConnection,
    workshop_id: u64,
    student_id: u64,
    is_teacher: bool,
) -> Result<Vec<WorkshopSubmission>, ()> {
    let submissions: Result<Vec<u64>, _> = submissions_t
        .filter(sub_workshop.eq(workshop_id).and(sub_student.eq(student_id)))
        .select(sub_id)
        .get_results(conn);
    if submissions.is_err() {
        return Err(());
    }
    let submissions = submissions.unwrap();
    for submission_id in submissions {
        if calculate_points(conn, submission_id).is_err() {
            return Err(());
        }
    }
    let submissions: Result<Vec<Submission>, _> = submissions_t
        .filter(sub_workshop.eq(workshop_id).and(sub_student.eq(student_id)))
        .get_results(conn);
    if submissions.is_err() {
        return Err(());
    }
    let submissions = submissions.unwrap();
    let submissions: Vec<WorkshopSubmission> = if is_teacher {
        submissions
            .into_iter()
            .map(|submission| {
                let no_reviews = if submission.meanpoints.is_none() {
                    true
                } else {
                    false
                };
                WorkshopSubmission {
                    id: submission.id,
                    title: submission.title,
                    date: submission.date,
                    locked: None,
                    student_id: Some(student_id),
                    reviews_done: submission.reviewsdone,
                    no_reviews,
                    points: submission.meanpoints,
                    max_points: submission.maxpoint,
                }
            })
            .collect()
    } else {
        submissions
            .into_iter()
            .map(|submission| {
                let no_reviews = if submission.meanpoints.is_none() {
                    true
                } else {
                    false
                };
                WorkshopSubmission {
                    id: submission.id,
                    title: submission.title,
                    date: submission.date,
                    locked: Some(submission.locked),
                    student_id: None,
                    reviews_done: submission.reviewsdone,
                    no_reviews,
                    points: submission.meanpoints,
                    max_points: submission.maxpoint,
                }
            })
            .collect()
    };
    Ok(submissions)
}

/// Get all workshop submissions.
/// Representation is adapted for teachers.
pub fn get_teacher_workshop_submissions(
    conn: &MysqlConnection,
    workshop_id: u64,
    student_id: u64,
) -> Result<Vec<WorkshopSubmission>, ()> {
    get_workshop_submissions_internal(conn, workshop_id, student_id, true)
}

/// Get all workshop submissions.
/// Representation is adapted for students.
pub fn get_student_workshop_submissions(
    conn: &MysqlConnection,
    workshop_id: u64,
    student_id: u64,
) -> Result<Vec<WorkshopSubmission>, ()> {
    get_workshop_submissions_internal(conn, workshop_id, student_id, false)
}

/// Calculate points of a submission.
/// Also closes all pending reviews.
fn calculate_points(conn: &MysqlConnection, submission_id: u64) -> Result<(), DbError> {
    // let submission = submissions_t
    //     .filter(
    //         sub_id.eq(submission_id).and(
    //             sub_reviews_done
    //                 .eq(true)
    //                 .and(sub_meanpoints.is_null().and(sub_error.eq(false))),
    //         ),
    //     )
    //     .first(conn);

    // Get submission past deadline with no calculated points
    let now = Local::now().naive_local();
    let submission = submissions_t
        .filter(
            sub_id
                .eq(submission_id)
                .and(sub_reviews_done.eq(false).and(sub_deadline.lt(now))),
        )
        .first(conn);
    if submission.is_err() {
        // Submission points are already calculated or not finished yet
        return Ok(());
    }

    // Submission was not processed yet
    // --------------------------------
    let mut submission: Submission = submission.unwrap();
    // If not already locked, lock it now
    if !submission.locked {
        let lock = db::submissions::lock(conn, submission_id);
        if lock.is_err() {
            return Err(DbError::new(
                DbErrorKind::UpdateFailed,
                "Submission Lock failed",
            ));
        }
        submission.locked = true;
    }

    // Close all reviews
    // -----------------
    let close = db::reviews::close_reviews(conn, submission_id);
    if close.is_err() {
        return Err(DbError::new(
            DbErrorKind::UpdateFailed,
            "Review Close failed",
        ));
    }
    // Get all reviews without errors
    let reviews = db::reviews::get_simple_review_points(conn, submission_id);
    if let Err(err) = reviews {
        return Err(err);
    }
    let reviews = reviews.unwrap();

    // Calculate points
    // ----------------
    let mut t_error: Result<(), DbError> = Ok(());
    let res = conn.transaction::<_, _, _>(|| {
        // If there are no reviews, a submissions cannot be graded
        if reviews.len() == 0 {
            // Save error state
            submission.reviewsdone = true;
            submission.error = true;
            let update = diesel::update(submissions_t.filter(sub_id.eq(submission.id)))
                .set(&submission)
                .execute(conn);
            if update.is_err() {
                return DbError::assign_and_rollback(
                    &mut t_error,
                    DbError::new(
                        DbErrorKind::UpdateFailed,
                        "Submission Error State Update failed",
                    ),
                );
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
                        Kind::Point => point.points * point.weight,
                        Kind::Grade => match point.points.round() as i64 {
                            1 => point_range * point.weight,
                            2 => point_range * 0.8 * point.weight,
                            3 => point_range * 0.6 * point.weight,
                            4 => point_range * 0.5 * point.weight,
                            _ => 0.0,
                        },
                        Kind::Percentage => point.points * point.weight,
                        Kind::Truefalse => match point.points.round() as i64 {
                            1 => point_range * point.weight,
                            _ => 0.0,
                        },
                    };
                    review_mean_points += weighted_points;
                }
                mean_points += review_mean_points;
            }
            mean_points /= reviews.len() as f64;
            // Update submission
            submission.reviewsdone = true;
            submission.maxpoint = Some(max_points);
            submission.meanpoints = Some(mean_points);
            let update = diesel::update(submissions_t.filter(sub_id.eq(submission.id)))
                .set(&submission)
                .execute(conn);
            if update.is_err() {
                return DbError::assign_and_rollback(
                    &mut t_error,
                    DbError::new(DbErrorKind::UpdateFailed, "Submission Update failed"),
                );
            }
        }
        Ok(())
    });

    if res.is_ok() {
        Ok(())
    } else {
        Err(t_error.err().unwrap_or(DbError::new(
            DbErrorKind::TransactionFailed,
            "Unknown error",
        )))
    }
}

/// Get review criteria for a submission.
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

/// Lock submission.
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

/// Get submission by submission id.
pub fn get_by_id(conn: &MysqlConnection, submission_id: u64) -> Result<Submission, Error> {
    submissions_t.filter(sub_id.eq(submission_id)).first(conn)
}
