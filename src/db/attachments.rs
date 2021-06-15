use crate::models::{Attachment, NewAttachment};
use crate::schema::attachments::dsl::{
    attachments as attachments_t, id as att_id, owner as att_owner,
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
