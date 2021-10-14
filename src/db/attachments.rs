//! CRUD operations for attachments.

use crate::db;
use crate::models::{Attachment, NewAttachment, SimpleAttachment};
use crate::schema::attachments::dsl::{
    attachments as attachments_t, id as att_id, owner as att_owner, title as att_title,
};
use crate::schema::submissionattachments::dsl::{
    attachment as subatt_att, submission as subatt_sub, submissionattachments as subatt_t,
};
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};

/// Create attachment.
pub fn create<'a>(conn: &MysqlConnection, title: String, owner: u64) -> Result<Attachment, ()> {
    let new_attachment = NewAttachment { title, owner };

    let att = conn.transaction::<Attachment, Error, _>(|| {
        let attachment_insert = diesel::insert_into(attachments_t)
            .values(&new_attachment)
            .execute(conn);
        if let Err(error) = attachment_insert {
            return Err(error);
        }
        let attachment: Attachment = attachments_t.order(att_id.desc()).first(conn).unwrap();
        Ok(attachment)
    });

    match att {
        Ok(att) => Ok(att),
        Err(_) => Err(()),
    }
}

/// Delete attachment.
pub fn delete(conn: &MysqlConnection, attachment_id: u64, user_id: u64) -> Result<(), ()> {
    // Check if attachment is already part of an submission
    let attachment = attachments_t
        .inner_join(subatt_t.on(att_id.eq(subatt_att)))
        .filter(att_id.eq(attachment_id).and(att_owner.eq(user_id)))
        .select(att_id)
        .first::<u64>(conn);
    if attachment.is_ok() {
        return Err(());
    }
    // If no, delete attachment
    let delete = conn.transaction::<_, _, _>(|| {
        let delete = diesel::delete(attachments_t.filter(att_id.eq(attachment_id))).execute(conn);
        if delete.is_err() {
            return Err(Error::RollbackTransaction);
        }
        Ok(())
    });

    match delete {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

/*
select a.id
    from attachments a
    inner join submissionattachments sb on a.id=sb.attachment
    where a.id = 1;
*/

/// Get attachment by attachment id.
pub fn get_by_id(conn: &MysqlConnection, id: u64) -> Result<Attachment, Error> {
    attachments_t.filter(att_id.eq(id)).first(conn)
}

/// Get all attachments from an user by its id.
#[allow(dead_code)]
pub fn get_by_user_id(conn: &MysqlConnection, user_id: u64) -> Result<Vec<Attachment>, Error> {
    attachments_t
        .filter(att_owner.eq(user_id))
        .get_results::<Attachment>(conn)
}

/// Get all attachment ids from an user by its id.
pub fn get_ids_by_user_id(conn: &MysqlConnection, user_id: u64) -> Result<Vec<u64>, Error> {
    attachments_t
        .select(att_id)
        .filter(att_owner.eq(user_id))
        .get_results::<u64>(conn)
}

// Get all attachments in simplified form from a submission.
fn get_by_submission_id_internal(
    conn: &MysqlConnection,
    submission_id: u64,
) -> Result<Vec<SimpleAttachment>, Error> {
    /*
    select a.id, a.title, a.owner
        from attachments a
        inner join submissionattachments sa on sa.attachment=a.id
        where sa.submission = 1;
     */
    attachments_t
        .inner_join(subatt_t.on(subatt_att.eq(att_id)))
        .filter(subatt_sub.eq(submission_id))
        .select((att_id, att_title))
        .get_results::<SimpleAttachment>(conn)
}

/// Get all attachments in simplified form from a submission.
pub fn get_by_submission_id(
    conn: &MysqlConnection,
    submission_id: u64,
) -> Result<Vec<SimpleAttachment>, Error> {
    get_by_submission_id_internal(conn, submission_id)
}

/// Get all attachments in simplified from from a submission
/// and lock the submission.
pub fn get_by_submission_id_and_lock_submission(
    conn: &MysqlConnection,
    submission_id: u64,
) -> Result<Vec<SimpleAttachment>, Error> {
    let lock = db::submissions::lock(conn, submission_id);
    if lock.is_err() {
        return Err(Error::NotFound);
    }
    get_by_submission_id_internal(conn, submission_id)
}
