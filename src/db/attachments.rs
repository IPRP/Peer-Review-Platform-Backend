use crate::models::{Attachment, NewAttachment};
use crate::schema::attachments::dsl::{attachments as attachments_t, id as att_id};
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
