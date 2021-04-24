/**use crate::models::{NewPost, Post};
use crate::schema::posts::dsl::{id, posts};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;

pub fn create_post<'a>(conn: &MysqlConnection, title: &'a str, body: &'a str) -> Post {
    let new_post = NewPost { title, body };

    diesel::insert_into(posts)
        .values(&new_post)
        .execute(conn)
        .expect("Error saving new post");

    posts.order(id.desc()).first(conn).unwrap()
}

pub fn create_post_event(conn: &MysqlConnection) {
    let _ = conn.batch_execute(
        r#"
        CREATE EVENT IF NOT EXISTS test_event_01
        ON SCHEDULE AT CURRENT_TIMESTAMP
        DO
          UPDATE posts
          SET title = 'TestRocket!', body = 'Greater'
          WHERE id = '2';
    "#,
    );
}*/
