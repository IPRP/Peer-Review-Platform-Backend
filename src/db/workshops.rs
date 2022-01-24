//! CRUD operations for workshops.

use crate::db;
use crate::db::error::{DbError, DbErrorKind};
use crate::db::models::*;
use crate::schema::criteria::dsl::{
    criteria as criteria_t, criterion as criteria_criterion, workshop as criteria_workshop,
};
use crate::schema::criterion::dsl::{criterion as criterion_t, id as c_id};
use crate::schema::users::dsl::{
    firstname as u_firstname, id as u_id, lastname as u_lastname, role as u_role, unit as u_unit,
    users as users_t,
};
use crate::schema::workshopattachments::dsl::{
    workshop as wsatt_ws, workshopattachments as wsatt_t,
};
use crate::schema::workshoplist::dsl::{
    role as wsl_role, user as wsl_user, workshop as wsl_ws, workshoplist as workshoplist_t,
};
use crate::schema::workshops::dsl::{
    anonymous as ws_anonymous, id as ws_id, reviewtimespan as ws_reviewtimespan,
    workshops as workshops_t,
};
use diesel::prelude::*;
use diesel::result::Error;

/// Get all workshops for an user.
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

/// Get workshop by submission id.
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

/// Get workshop by review id.
#[allow(dead_code)]
pub fn get_by_review_id(conn: &MysqlConnection, review_id: u64) -> Result<Workshop, Error> {
    let review = db::reviews::get_by_id(conn, review_id);
    if review.is_err() {
        return Err(Error::NotFound);
    }
    let review = review.unwrap();
    workshops_t.filter(ws_id.eq(review.workshop)).first(conn)
}

/// Create new workshop.
pub fn create<'a>(
    conn: &MysqlConnection,
    teacher_id: u64,
    title: String,
    content: String,
    end: chrono::NaiveDateTime,
    review_timespan: i64,
    anonymous: bool,
    teachers: Vec<u64>,
    students: Vec<u64>,
    criteria: Vec<NewCriterion>,
    attachments: Vec<u64>,
) -> Result<Workshop, DbError> {
    let new_workshop = NewWorkshop {
        title,
        content,
        end,
        reviewtimespan: review_timespan,
        anonymous,
    };

    let mut t_error: Result<(), DbError> = Ok(());
    let ws = conn.transaction::<Workshop, _, _>(|| {
        // Filter students & teachers
        let students = users_t
            .filter(u_role.eq(Role::Student).and(u_id.eq_any(students)))
            .get_results::<User>(conn);
        if students.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::ReadFailed, "Could not determine Students"),
            );
        }
        let students = students.unwrap();
        let teachers = users_t
            .filter(u_role.eq(Role::Teacher).and(u_id.eq_any(teachers)))
            .get_results::<User>(conn);
        if teachers.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::ReadFailed, "Could not determine Teachers"),
            );
        }
        let mut teachers = teachers.unwrap();
        // Insert criteria
        let criterion_insert = diesel::insert_into(criterion_t)
            .values(&criteria)
            .execute(conn);
        if criterion_insert.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::CreateFailed, "Could not insert Criteria"),
            );
        }
        let mut last_criterion_id = criterion_t
            .select(c_id)
            .order(c_id.desc())
            .first(conn)
            .unwrap();
        last_criterion_id += 1;
        let first_criterion_id = last_criterion_id - criteria.len() as u64;
        let criterion_ids: Vec<u64> = (first_criterion_id..last_criterion_id).collect();
        // Insert workshop
        let insert = diesel::insert_into(workshops_t)
            .values(&new_workshop)
            .execute(conn);
        if insert.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::CreateFailed, "Could not insert Workshop"),
            );
        }
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
        let workshop_insert = diesel::insert_into(workshoplist_t)
            .values(&new_workshoplist)
            .execute(conn);
        if workshop_insert.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::CreateFailed, "Could not insert Workshoplist"),
            );
        }
        // Assign criteria to workshop
        let new_criteria = criterion_ids
            .into_iter()
            .map(|c| Criteria {
                workshop: workshop.id,
                criterion: c,
            })
            .collect::<Vec<Criteria>>();
        let criteria_insert = diesel::insert_into(criteria_t)
            .values(&new_criteria)
            .execute(conn);
        if criteria_insert.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(
                    DbErrorKind::CreateFailed,
                    "Could not insert Workshop-Criteria",
                ),
            );
        }
        // Relate attachments to workshop
        let all_teacher_attachments = db::attachments::get_ids_by_user_id(conn, teacher_id);
        if all_teacher_attachments.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::ReadFailed, "Could not get Attachments"),
            );
        }
        let all_teacher_attachments = all_teacher_attachments.unwrap();
        let workshop_attachments: Vec<Workshopattachment> = attachments
            .into_iter()
            .filter_map(|att_id| {
                if all_teacher_attachments.contains(&att_id) {
                    Some(Workshopattachment {
                        workshop: workshop.id,
                        attachment: att_id,
                    })
                } else {
                    None
                }
            })
            .collect();
        let attachment_insert = diesel::insert_into(wsatt_t)
            .values(&workshop_attachments)
            .execute(conn);
        if attachment_insert.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::ReadFailed, "Could not insert Attachments"),
            );
        }
        Ok(workshop)
    });
    match ws {
        Ok(ws) => Ok(ws),
        Err(_) => Err(t_error.err().unwrap_or(DbError::new(
            DbErrorKind::TransactionFailed,
            "Unknown error",
        ))),
    }
}

/// Update existing workshop.
pub fn update(
    conn: &MysqlConnection,
    teacher_id: u64,
    workshop_id: u64,
    title: String,
    content: String,
    end: chrono::NaiveDateTime,
    review_timespan: i64,
    teachers: Vec<u64>,
    students: Vec<u64>,
    criteria: Vec<NewCriterion>,
    attachments: Vec<u64>,
) -> Result<Workshop, DbError> {
    let workshop = workshops_t.filter(ws_id.eq(workshop_id)).first(conn);
    if workshop.is_err() {
        return Err(DbError::new(
            DbErrorKind::ReadFailed,
            format!("Could not find Workshop with Id {}", workshop_id),
        ));
    }
    let mut workshop: Workshop = workshop.unwrap();
    workshop.title = title;
    workshop.content = content;
    workshop.end = end;
    workshop.reviewtimespan = review_timespan;

    let mut t_error: Result<(), DbError> = Ok(());
    let ws = conn.transaction::<Workshop, _, _>(|| {
        // Remove student & teachers
        let delete = diesel::delete(workshoplist_t.filter(wsl_ws.eq(workshop_id))).execute(conn);
        if delete.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(
                    DbErrorKind::DeleteFailed,
                    "Could not delete Students & Teachers",
                ),
            );
        }
        // Remove criteria
        let delete =
            diesel::delete(criteria_t.filter(criteria_workshop.eq(workshop_id))).execute(conn);
        if delete.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::DeleteFailed, "Could not delete Criteria"),
            );
        }
        // Remove attachments
        let delete = diesel::delete(wsatt_t.filter(wsatt_ws.eq(workshop_id))).execute(conn);
        if delete.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::DeleteFailed, "Could not delete Attachments"),
            );
        }
        // Filter students & teachers
        let students = users_t
            .filter(u_role.eq(Role::Student).and(u_id.eq_any(students)))
            .get_results::<User>(conn);
        if students.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::ReadFailed, "Could not determine Students"),
            );
        }
        let students = students.unwrap();
        let teachers = users_t
            .filter(u_role.eq(Role::Teacher).and(u_id.eq_any(teachers)))
            .get_results::<User>(conn);
        if teachers.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::ReadFailed, "Could not determine Teachers"),
            );
        }
        let mut teachers = teachers.unwrap();

        // Insert criteria
        let insert = diesel::insert_into(criterion_t)
            .values(&criteria)
            .execute(conn);
        if insert.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::CreateFailed, "Could not insert Criteria"),
            );
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
        let update = diesel::update(workshops_t.filter(ws_id.eq(workshop.id)))
            .set(&workshop)
            .execute(conn);
        if update.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::UpdateFailed, "Could not update Workshop"),
            );
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
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::CreateFailed, "Could not insert Workshoplist"),
            );
        }

        // Assign criteria to workshop
        let new_criteria = criterion_ids
            .into_iter()
            .map(|c| Criteria {
                workshop: workshop.id,
                criterion: c,
            })
            .collect::<Vec<Criteria>>();
        let insert = diesel::insert_into(criteria_t)
            .values(&new_criteria)
            .execute(conn);
        if insert.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(
                    DbErrorKind::CreateFailed,
                    "Could not insert Workshop-Criteria",
                ),
            );
        }

        // Relate attachments to workshop
        let all_teacher_attachments = db::attachments::get_ids_by_user_id(conn, teacher_id);
        if all_teacher_attachments.is_err() {
            return Err(Error::RollbackTransaction);
        }
        let all_teacher_attachments = all_teacher_attachments.unwrap();
        let workshop_attachments: Vec<Workshopattachment> = attachments
            .into_iter()
            .filter_map(|att_id| {
                if all_teacher_attachments.contains(&att_id) {
                    Some(Workshopattachment {
                        workshop: workshop.id,
                        attachment: att_id,
                    })
                } else {
                    None
                }
            })
            .collect();
        let attachment_insert = diesel::insert_into(wsatt_t)
            .values(&workshop_attachments)
            .execute(conn);
        if attachment_insert.is_err() {
            return DbError::assign_and_rollback(
                &mut t_error,
                DbError::new(DbErrorKind::ReadFailed, "Could not insert Attachments"),
            );
        }
        Ok(workshop)
    });

    match ws {
        Ok(ws) => Ok(ws),
        Err(_) => Err(t_error.err().unwrap_or(DbError::new(
            DbErrorKind::TransactionFailed,
            "Unknown error",
        ))),
    }
}

/// Delete existing workshop.
pub fn delete(conn: &MysqlConnection, id: u64) -> Result<(), ()> {
    let workshop: Result<Workshop, diesel::result::Error> =
        workshops_t.filter(ws_id.eq(id)).first(conn);
    if workshop.is_ok() {
        let workshop_delete = diesel::delete(workshops_t.filter(ws_id.eq(id))).execute(conn);
        if workshop_delete.is_ok() {
            Ok(())
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

/// Check if student is part of a workshop.
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

// Gets students/teachers of a workshop.
fn roles_in_workshop(
    conn: &MysqlConnection,
    workshop_id: u64,
    role: Role,
    is_teacher: bool,
) -> Result<Vec<WorkshopUser>, ()> {
    let users = workshoplist_t
        .inner_join(users_t.on(u_id.eq(wsl_user)))
        .filter(wsl_ws.eq(workshop_id).and(u_role.eq(role.clone())))
        .select((u_id, u_firstname, u_lastname, u_unit))
        .get_results::<(u64, String, String, Option<String>)>(conn);
    if users.is_err() {
        return Err(());
    }
    let users: Vec<(u64, String, String, Option<String>)> = users.unwrap();
    let users = users
        .into_iter()
        .map(|user| {
            let submissions = if role == Role::Student {
                if is_teacher {
                    let submissions = db::submissions::get_teacher_workshop_submissions(
                        conn,
                        workshop_id,
                        user.0,
                    );
                    match submissions {
                        Ok(submissions) => Some(submissions),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            };
            WorkshopUser {
                id: user.0,
                firstname: user.1,
                lastname: user.2,
                group: user.3,
                submissions,
            }
        })
        .collect();

    Ok(users)
}

/// Gets students of a workshop.
#[allow(dead_code)]
pub fn students_in_workshop(
    conn: &MysqlConnection,
    workshop_id: u64,
    is_teacher: bool,
) -> Result<Vec<WorkshopUser>, ()> {
    roles_in_workshop(conn, workshop_id, Role::Student, is_teacher)
}

/// Gets teachers of a workshop.
#[allow(dead_code)]
pub fn teachers_in_workshop(
    conn: &MysqlConnection,
    workshop_id: u64,
    is_teacher: bool,
) -> Result<Vec<WorkshopUser>, ()> {
    roles_in_workshop(conn, workshop_id, Role::Teacher, is_teacher)
}

/// Get teacher workshop by workshop id.
pub fn get_teacher_workshop(
    conn: &MysqlConnection,
    workshop_id: u64,
) -> Result<TeacherWorkshop, ()> {
    let workshop: Result<Workshop, diesel::result::Error> =
        workshops_t.filter(ws_id.eq(workshop_id)).first(conn);
    if workshop.is_err() {
        return Err(());
    }
    let workshop = workshop.unwrap();
    let students = roles_in_workshop(conn, workshop_id, Role::Student, true);
    if students.is_err() {
        return Err(());
    }
    let students = students.unwrap();
    let teachers = roles_in_workshop(conn, workshop_id, Role::Teacher, true);
    if teachers.is_err() {
        return Err(());
    }
    let teachers = teachers.unwrap();

    let criteria: Result<Vec<u64>, _> = criteria_t
        .filter(criteria_workshop.eq(workshop_id))
        .select(criteria_criterion)
        .get_results(conn);
    if criteria.is_err() {
        return Err(());
    }
    let criteria = criteria.unwrap();

    let criteria: Result<Vec<Criterion>, _> =
        criterion_t.filter(c_id.eq_any(criteria)).get_results(conn);
    if criteria.is_err() {
        return Err(());
    }
    let criteria = criteria.unwrap();

    Ok(TeacherWorkshop {
        title: workshop.title,
        content: workshop.content,
        end: workshop.end,
        review_timespan: workshop.reviewtimespan,
        anonymous: workshop.anonymous,
        students,
        teachers,
        criteria,
    })
}

/// Get student workshop by workshop id.
pub fn get_student_workshop(
    conn: &MysqlConnection,
    workshop_id: u64,
    student_id: u64,
) -> Result<StudentWorkshop, ()> {
    if !student_in_workshop(conn, student_id, workshop_id) {
        return Err(());
    }

    let workshop: Result<Workshop, diesel::result::Error> =
        workshops_t.filter(ws_id.eq(workshop_id)).first(conn);
    if workshop.is_err() {
        return Err(());
    }
    let workshop = workshop.unwrap();
    let students = roles_in_workshop(conn, workshop_id, Role::Student, false);
    if students.is_err() {
        return Err(());
    }
    let students = students.unwrap();
    let teachers = roles_in_workshop(conn, workshop_id, Role::Teacher, false);
    if teachers.is_err() {
        return Err(());
    }
    let teachers = teachers.unwrap();
    let submissions =
        db::submissions::get_student_workshop_submissions(conn, workshop_id, student_id);
    if submissions.is_err() {
        return Err(());
    }
    let submissions = submissions.unwrap();
    let reviews = db::reviews::get_student_workshop_reviews(conn, workshop_id, student_id);
    if reviews.is_err() {
        return Err(());
    }
    let reviews = reviews.unwrap();

    Ok(StudentWorkshop {
        title: workshop.title,
        content: workshop.content,
        end: workshop.end,
        anonymous: workshop.anonymous,
        students,
        teachers,
        submissions,
        reviews,
    })
}

/// Get criteria ids for given workshop.
pub fn get_criteria(conn: &MysqlConnection, workshop_id: u64) -> Result<Vec<u64>, Error> {
    criteria_t
        .select(criteria_criterion)
        .filter(criteria_workshop.eq(workshop_id))
        .get_results::<u64>(conn)
}

/// Check if workshop is anonymous or not.
pub fn is_anonymous(conn: &MysqlConnection, workshop_id: u64) -> bool {
    let anonymous = workshops_t
        .select(ws_anonymous)
        .filter(ws_id.eq(workshop_id))
        .first(conn);
    if anonymous.is_err() {
        return false;
    }
    let anonymous = anonymous.unwrap();
    anonymous
}

/// Get review timespan
pub fn get_review_timespan(
    conn: &MysqlConnection,
    workshop_id: u64,
) -> Result<chrono::Duration, DbError> {
    let minutes: QueryResult<i64> = workshops_t
        .select(ws_reviewtimespan)
        .filter(ws_id.eq(workshop_id))
        .first(conn);
    if minutes.is_err() {
        return Err(DbError::new(
            DbErrorKind::ReadFailed,
            format!("Review Timespan for Workshop {} not found ", workshop_id),
        ));
    }
    let review_timespan = chrono::Duration::minutes(minutes.unwrap());
    Ok(review_timespan)
}
