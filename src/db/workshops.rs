use crate::models::{
    Criteria as NewCriteria, NewCriterion, NewStudent, NewWorkshop, Role, User, Workshop,
    Workshoplist,
};
use crate::schema::criteria::dsl::criteria as criteria_t;
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
