use crate::db;
use crate::models::{
    Criteria as NewCriteria, NewCriterion, NewStudent, NewWorkshop, Role, Submission, User,
    Workshop, Workshoplist,
};
use crate::schema::criteria::dsl::{
    criteria as criteria_t, criterion as criteria_criterion, workshop as criteria_workshop,
};
use crate::schema::criterion::dsl::{criterion as criterion_t, id as c_id};
use crate::schema::users::dsl::{id as u_id, role as u_role, users as users_t};
use crate::schema::workshoplist::dsl::{
    role as wsl_role, user as wsl_user, workshop as wsl_ws, workshoplist as workshoplist_t,
};
use crate::schema::workshops::dsl::{
    anonymous as ws_anonymous, content as ws_content, end as ws_end, id as ws_id,
    title as ws_title, workshops as workshops_t,
};
use diesel::prelude::*;
use diesel::result::Error;

pub fn get_by_user(conn: &MysqlConnection, id: u64) -> Vec<Workshop> {
    let workshop_ids = workshoplist_t
        .select(wsl_ws)
        .filter(wsl_user.eq(id))
        .get_results::<u64>(conn);
    match workshop_ids {
        Ok(workshop_ids) => {
            let workshops = workshops_t
                .filter(ws_id.eq_any(workshop_ids))
                .get_results::<Workshop>(conn);
            match workshops {
                Ok(workshops) => workshops,
                Err(_) => Vec::with_capacity(0),
            }
        }
        Err(_) => Vec::with_capacity(0),
    }
}

pub fn get_by_submission_id(conn: &MysqlConnection, submission_id: u64) -> Result<Workshop, Error> {
    let submission = db::submissions::get_by_id(conn, submission_id);
    if submission.is_err() {
        return Err(Error::NotFound);
    }
    let submission = submission.unwrap();
    workshops_t
        .filter(ws_id.eq(submission.workshop))
        .first(conn)
}

pub fn get_by_review_id(conn: &MysqlConnection, review_id: u64) -> Result<Workshop, Error> {
    let review = db::reviews::get_by_id(conn, review_id);
    if review.is_err() {
        return Err(Error::NotFound);
    }
    let review = review.unwrap();
    workshops_t.filter(ws_id.eq(review.workshop)).first(conn)
}

pub fn create<'a>(
    conn: &MysqlConnection,
    title: String,
    content: String,
    end: chrono::NaiveDateTime,
    anonymous: bool,
    teachers: Vec<u64>,
    students: Vec<u64>,
    criteria: Vec<NewCriterion>,
) -> Result<Workshop, ()> {
    let new_workshop = NewWorkshop {
        title,
        content,
        end,
        anonymous,
    };
    let ws = conn.transaction::<Workshop, _, _>(|| {
        // Filter students & teachers
        let students = users_t
            .filter(u_role.eq(Role::Student).and(u_id.eq_any(students)))
            .get_results::<User>(conn);
        if students.is_err() {
            return Err(Error::RollbackTransaction);
        }
        let students = students.unwrap();
        println!("{:?}", students);
        let teachers = users_t
            .filter(u_role.eq(Role::Teacher).and(u_id.eq_any(teachers)))
            .get_results::<User>(conn);
        if teachers.is_err() {
            return Err(Error::RollbackTransaction);
        }
        let mut teachers = teachers.unwrap();
        println!("{:?}", teachers);
        // Insert criteria
        diesel::insert_into(criterion_t)
            .values(&criteria)
            .execute(conn);
        let mut last_criterion_id = criterion_t
            .select(c_id)
            .order(c_id.desc())
            .first(conn)
            .unwrap();
        last_criterion_id += 1;
        let first_criterion_id = last_criterion_id - criteria.len() as u64;
        let criterion_ids: Vec<u64> = (first_criterion_id..last_criterion_id).collect();
        println!("{:?}", criterion_ids);
        // Insert workshop
        diesel::insert_into(workshops_t)
            .values(&new_workshop)
            .execute(conn)
            .expect("Error saving new workshop");
        let workshop: Workshop = workshops_t.order(ws_id.desc()).first(conn).unwrap();
        // Assign students & teachers to workshop
        let mut new_workshoplist = students;
        new_workshoplist.append(&mut teachers);
        let new_workshoplist = new_workshoplist
            .into_iter()
            .map(|u| Workshoplist {
                workshop: workshop.id,
                user: u.id,
                role: u.role,
            })
            .collect::<Vec<Workshoplist>>();
        diesel::insert_into(workshoplist_t)
            .values(&new_workshoplist)
            .execute(conn);
        // Assign criteria to workshop
        let new_criteria = criterion_ids
            .into_iter()
            .map(|c| NewCriteria {
                workshop: workshop.id,
                criterion: c,
            })
            .collect::<Vec<NewCriteria>>();
        diesel::insert_into(criteria_t)
            .values(&new_criteria)
            .execute(conn);
        Ok(workshop)
    });
    match ws {
        Ok(ws) => Ok(ws),
        Err(_) => Err(()),
    }
}

pub fn update(
    conn: &MysqlConnection,
    workshop_id: u64,
    title: String,
    content: String,
    end: chrono::NaiveDateTime,
    teachers: Vec<u64>,
    students: Vec<u64>,
    criteria: Vec<NewCriterion>,
) -> Result<Workshop, ()> {
    let workshop = workshops_t.filter(ws_id.eq(workshop_id)).first(conn);
    if workshop.is_err() {
        return Err(());
    }
    let mut workshop: Workshop = workshop.unwrap();
    workshop.title = title;
    workshop.content = content;
    workshop.end = end;
    let ws = conn.transaction::<Workshop, _, _>(|| {
        // Remove student & teachers
        let delete = diesel::delete(workshoplist_t.filter(wsl_ws.eq(workshop_id))).execute(conn);
        if delete.is_err() {
            return Err(Error::RollbackTransaction);
        }
        // Remove criteria
        let delete =
            diesel::delete(criteria_t.filter(criteria_workshop.eq(workshop_id))).execute(conn);
        if delete.is_err() {
            return Err(Error::RollbackTransaction);
        }

        // Filter students & teachers
        let students = users_t
            .filter(u_role.eq(Role::Student).and(u_id.eq_any(students)))
            .get_results::<User>(conn);
        if students.is_err() {
            return Err(Error::RollbackTransaction);
        }
        let students = students.unwrap();
        let teachers = users_t
            .filter(u_role.eq(Role::Teacher).and(u_id.eq_any(teachers)))
            .get_results::<User>(conn);
        if teachers.is_err() {
            return Err(Error::RollbackTransaction);
        }
        let mut teachers = teachers.unwrap();

        // Insert criteria
        let insert = diesel::insert_into(criterion_t)
            .values(&criteria)
            .execute(conn);
        if insert.is_err() {
            return Err(Error::RollbackTransaction);
        }

        let mut last_criterion_id = criterion_t
            .select(c_id)
            .order(c_id.desc())
            .first(conn)
            .unwrap();
        last_criterion_id += 1;
        let first_criterion_id = last_criterion_id - criteria.len() as u64;
        let criterion_ids: Vec<u64> = (first_criterion_id..last_criterion_id).collect();

        // Update workshop
        // diesel::update(reviews_t).set(&review).execute(conn);
        let update = diesel::update(workshops_t.filter(ws_id.eq(workshop.id)))
            .set(&workshop)
            .execute(conn);
        if update.is_err() {
            return Err(Error::RollbackTransaction);
        }

        // Assign students & teachers to workshop
        let mut new_workshoplist = students;
        new_workshoplist.append(&mut teachers);
        let new_workshoplist = new_workshoplist
            .into_iter()
            .map(|u| Workshoplist {
                workshop: workshop.id,
                user: u.id,
                role: u.role,
            })
            .collect::<Vec<Workshoplist>>();
        let insert = diesel::insert_into(workshoplist_t)
            .values(&new_workshoplist)
            .execute(conn);
        if insert.is_err() {
            return Err(Error::RollbackTransaction);
        }

        // Assign criteria to workshop
        let new_criteria = criterion_ids
            .into_iter()
            .map(|c| NewCriteria {
                workshop: workshop.id,
                criterion: c,
            })
            .collect::<Vec<NewCriteria>>();
        let insert = diesel::insert_into(criteria_t)
            .values(&new_criteria)
            .execute(conn);
        if insert.is_err() {
            return Err(Error::RollbackTransaction);
        }

        Ok(workshop)
    });

    match ws {
        Ok(ws) => Ok(ws),
        Err(_) => Err(()),
    }
}

pub fn delete(conn: &MysqlConnection, id: u64) -> Result<(), ()> {
    let workshop: Result<Workshop, diesel::result::Error> =
        workshops_t.filter(ws_id.eq(id)).first(conn);
    if workshop.is_ok() {
        diesel::delete(workshops_t.filter(ws_id.eq(id))).execute(conn);
        Ok(())
    } else {
        Err(())
    }
}

pub fn student_in_workshop(conn: &MysqlConnection, student_id: u64, workshop_id: u64) -> bool {
    let exists: Result<Workshoplist, diesel::result::Error> = workshoplist_t
        .filter(
            wsl_ws
                .eq(workshop_id)
                .and(wsl_user.eq(student_id).and(wsl_role.eq(Role::Student))),
        )
        .first(conn);
    if exists.is_ok() {
        true
    } else {
        false
    }
}

pub fn get_criteria(conn: &MysqlConnection, workshop_id: u64) -> Result<Vec<u64>, Error> {
    criteria_t
        .select(criteria_criterion)
        .filter(criteria_workshop.eq(workshop_id))
        .get_results::<u64>(conn)
}
