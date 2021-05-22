use crate::models::{
    Criteria as NewCriteria, NewCriterion, NewStudent, NewWorkshop, Role, User, Workshop,
    Workshoplist,
};
use crate::schema::criteria::dsl::criteria as criteria_t;
use crate::schema::criterion::dsl::{criterion, id as c_id};
use crate::schema::users::dsl::{id as u_id, role as u_role, users};
use crate::schema::workshoplist::dsl::workshoplist;
use crate::schema::workshops::dsl::{
    anonymous as dsl_anonymous, content as dsl_content, end as dls_end, id as dsl_id,
    title as dsl_title, workshops,
};
use diesel::prelude::*;
use diesel::result::Error;

pub fn create_workshop<'a>(
    conn: &MysqlConnection,
    title: String,
    content: String,
    end: chrono::NaiveDate,
    anonymous: bool,
    teachers: Vec<u64>,
    students: Vec<u64>,
    criteria: Vec<NewCriterion>,
) -> Result<Workshop, &'static str> {
    let new_workshop = NewWorkshop {
        title,
        content,
        end,
        anonymous,
    };
    // Filter students & teachers
    let students = users
        .filter(u_role.eq(Role::Student).and(u_id.eq_any(students)))
        .get_results::<User>(conn);
    if students.is_err() {
        todo!();
    }
    let students = students.unwrap();
    println!("{:?}", students);
    let teachers = users
        .filter(u_role.eq(Role::Teacher).and(u_id.eq_any(teachers)))
        .get_results::<User>(conn);
    if teachers.is_err() {
        todo!();
    }
    let mut teachers = teachers.unwrap();
    println!("{:?}", teachers);
    // Insert criteria
    diesel::insert_into(criterion)
        .values(&criteria)
        .execute(conn);
    let mut last_criterion_id = criterion
        .select(c_id)
        .order(c_id.desc())
        .first(conn)
        .unwrap();
    last_criterion_id += 1;
    let first_criterion_id = last_criterion_id - criteria.len() as u64;
    let criterion_ids: Vec<u64> = (first_criterion_id..last_criterion_id).collect();
    println!("{:?}", criterion_ids);
    // Insert workshop
    diesel::insert_into(workshops)
        .values(&new_workshop)
        .execute(conn)
        .expect("Error saving new workshop");
    let workshop: Workshop = workshops.order(dsl_id.desc()).first(conn).unwrap();
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
    diesel::insert_into(workshoplist)
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
}
