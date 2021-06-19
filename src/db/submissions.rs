use crate::db;
use crate::models::{
    Criterion, NewSubmission, Role, SimpleAttachment, Submission, Submissionattachment,
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
    error as sub_error, id as sub_id, meanpoints as sub_meanpoints, student as sub_student,
    submissions as submissions_t,
};
use diesel::prelude::*;
use diesel::result::Error;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<i64>,
    #[serde(rename(serialize = "maxPoints"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_points: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firstname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastname: Option<String>,
}

pub fn get_own_submission(
    conn: &MysqlConnection,
    submission_id: u64,
    user_id: u64,
) -> Result<OwnSubmission, ()> {
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

    // TODO reviews

    Ok(OwnSubmission {
        title: submission.title,
        comment: submission.comment,
        attachments,
        locked: submission.locked,
        date: submission.date,
        reviews_done: submission.reviewsdone,
        points: None,
        max_points: None,
        firstname: None,
        lastname: None,
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
    let attachments = db::attachments::get_by_submission_id(conn, submission_id);
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
            sub_id
                .eq(submission_id)
                .and(sub_meanpoints.is_null().and(sub_error.eq(false))),
        )
        .first();

    if submission.is_err() {
        // Submission points are already calculated
        return Ok(());
    }
    let submission: Submission = submission.unwrap();
    // Get all reviews without errors

    // If there are not, this review cannot be graded (no reviews)

    // Calculate mean points and max points (based on criterion and weights)

    // Update submission

    Ok(())
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
