use crate::db::workshops::student_in_workshop;
use crate::models::{NewSubmission, Submission, Submissionattachment};
use crate::schema::criteria::dsl::workshop;
use crate::schema::submissionattachments::dsl::{
    attachment as subatt_att, submission as subatt_sub, submissionattachments as subatt_t,
};
use crate::schema::submissions::dsl::{id as sub_id, submissions as submissions_t};
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
    if !student_in_workshop(conn, student_id, workshop_id) {
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
        let subatt: Vec<Submissionattachment> = attachments
            .into_iter()
            .map(|att_id| Submissionattachment {
                submission: submission.id,
                attachment: att_id,
            })
            .collect();
        diesel::insert_into(subatt_t).values(&subatt).execute(conn);

        // Relate criteria to submission

        // Assign reviews

        Ok(submission)
    });

    match submission {
        Ok(submission) => Ok(submission),
        Err(_) => Err(()),
    }
}
