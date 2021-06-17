use crate::models::{Attachment, NewAttachment, SimpleAttachment};
use crate::schema::attachments::dsl::{
    attachments as attachments_t, id as att_id, owner as att_owner, title as att_title,
};
use crate::schema::submissionattachments::dsl::{
    attachment as subatt_att, submission as subatt_sub, submissionattachments as subatt_t,
};
use diesel::prelude::*;
use diesel::result::Error;

pub fn create<'a>(conn: &MysqlConnection, title: String, owner: u64) -> Result<Attachment, ()> {
    let new_attachment = NewAttachment { title, owner };

    let att = conn.transaction::<Attachment, Error, _>(|| {
        diesel::insert_into(attachments_t)
            .values(&new_attachment)
            .execute(conn);
        let attachment: Attachment = attachments_t.order(att_id.desc()).first(conn).unwrap();
        Ok(attachment)
    });

    match att {
        Ok(att) => Ok(att),
        Err(_) => Err(()),
    }
}

pub fn get_by_id(conn: &MysqlConnection, id: u64) -> Result<Attachment, Error> {
    attachments_t.filter(att_id.eq(id)).first(conn)
}

pub fn get_by_user_id(conn: &MysqlConnection, user_id: u64) -> Result<Vec<Attachment>, Error> {
    attachments_t
        .filter(att_owner.eq(user_id))
        .get_results::<Attachment>(conn)
}

pub fn get_ids_by_user_id(conn: &MysqlConnection, user_id: u64) -> Result<Vec<u64>, Error> {
    attachments_t
        .select(att_id)
        .filter(att_owner.eq(user_id))
        .get_results::<u64>(conn)
}

pub fn get_by_submission_id(
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
