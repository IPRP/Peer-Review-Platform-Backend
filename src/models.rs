use crate::schema::*;

// For Future: Patch file (MySQL Enum to RoleMapping)
// See: http://diesel.rs/guides/configuring-diesel-cli.html
// And: https://github.com/adwhit/diesel-derive-enum/issues/56
// And: https://github.com/diesel-rs/diesel/issues/2154

#[derive(DbEnum, Clone, Debug, PartialEq)]
pub enum Role {
    Student,
    Teacher,
}

impl Role {
    pub fn to_string(&self) -> String {
        match self {
            Role::Student => String::from("student"),
            Role::Teacher => String::from("teacher"),
        }
    }
}

#[derive(Debug, Queryable, Clone)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub password: String,
    pub role: Role,
    pub unit: Option<String>,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewStudent {
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub password: String,
    pub role: Role,
    pub unit: String,
}

impl NewStudent {
    pub fn new(
        username: String,
        firstname: String,
        lastname: String,
        password: String,
        unit: String,
    ) -> Self {
        let role = Role::Student;
        NewStudent {
            username,
            firstname,
            lastname,
            password,
            role,
            unit,
        }
    }
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewTeacher {
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub password: String,
    pub role: Role,
}

impl NewTeacher {
    pub fn new(username: String, firstname: String, lastname: String, password: String) -> Self {
        let role = Role::Teacher;
        NewTeacher {
            username,
            firstname,
            lastname,
            password,
            role,
        }
    }
}

#[derive(Queryable, AsChangeset, Clone)]
pub struct Workshop {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub end: chrono::NaiveDateTime,
    pub anonymous: bool,
}

#[derive(Insertable)]
#[table_name = "workshops"]
pub struct NewWorkshop {
    pub title: String,
    pub content: String,
    pub end: chrono::NaiveDateTime,
    pub anonymous: bool,
}

#[derive(Insertable, Queryable, Clone)]
#[table_name = "criteria"]
pub struct Criteria {
    pub workshop: u64,
    pub criterion: u64,
}

#[derive(DbEnum, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Point,
    Grade,
    Percentage,
    Truefalse,
}

impl Kind {
    pub fn from(str: &str) -> Result<Self, String> {
        let input = str.to_lowercase();
        if input.eq("point") {
            Ok(Kind::Point)
        } else if input.eq("grade") {
            Ok(Kind::Grade)
        } else if input.eq("percentage") {
            Ok(Kind::Percentage)
        } else if input.eq("truefalse") {
            Ok(Kind::Truefalse)
        } else {
            Err(String::new())
        }
    }
}

#[derive(Queryable, Clone, Serialize)]
pub struct Criterion {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub weight: f64,
    #[serde(rename = "type")]
    pub kind: Kind,
}

#[derive(Insertable)]
#[table_name = "criterion"]
pub struct NewCriterion {
    pub title: String,
    pub content: String,
    pub weight: f64,
    pub kind: Kind,
}

#[derive(Insertable, Queryable, Clone)]
#[table_name = "workshoplist"]
pub struct Workshoplist {
    pub workshop: u64,
    pub user: u64,
    pub role: Role,
}

#[derive(Queryable, Clone)]
pub struct Attachment {
    pub id: u64,
    pub title: String,
    pub owner: Option<u64>,
}

#[derive(Queryable, Clone, Serialize)]
pub struct SimpleAttachment {
    pub id: u64,
    pub title: String,
}

#[derive(Insertable, Queryable, Clone)]
#[table_name = "attachments"]
pub struct NewAttachment {
    pub title: String,
    pub owner: u64,
}

/*
CREATE TABLE submissions
(
    id          SERIAL PRIMARY KEY,
    title       VARCHAR(255)    NOT NULL,
    comment     TEXT            NOT NULL,
    student     BIGINT UNSIGNED,
    workshop    BIGINT UNSIGNED NOT NULL,
    date        DATETIME        NOT NULL,
    locked      BOOL            NOT NULL,
    reviewsdone BOOL            NOT NULL,
    error       BOOl            NOT NULL,
    meanpoints  DOUBLE,
    maxpoint    DOUBLE,
    FOREIGN KEY (student) REFERENCES users (id) ON DELETE SET NULL,
    FOREIGN KEY (workshop) REFERENCES workshops (id) ON DELETE CASCADE
);

CREATE TABLE submissioncriteria
(
    submission BIGINT UNSIGNED NOT NULL,
    criterion  BIGINT UNSIGNED NOT NULL,
    PRIMARY KEY (submission, criterion),
    FOREIGN KEY (submission) REFERENCES submissions (id) ON DELETE CASCADE,
    FOREIGN KEY (criterion) REFERENCES criterion (id) ON DELETE CASCADE
);

CREATE TABLE submissionattachments
(
    submission BIGINT UNSIGNED NOT NULL,
    attachment BIGINT UNSIGNED NOT NULL,
    PRIMARY KEY (submission, attachment),
    FOREIGN KEY (submission) REFERENCES submissions (id) ON DELETE CASCADE,
    FOREIGN KEY (attachment) REFERENCES attachments (id) ON DELETE CASCADE
); */

#[derive(Queryable, AsChangeset, Clone)]
pub struct Submission {
    pub id: u64,
    pub title: String,
    pub comment: String,
    pub student: Option<u64>,
    pub workshop: u64,
    pub date: chrono::NaiveDateTime,
    pub locked: bool,
    pub reviewsdone: bool,
    pub error: bool,
    pub meanpoints: Option<f64>,
    pub maxpoint: Option<f64>,
}

#[derive(Insertable, Queryable, Clone)]
#[table_name = "submissions"]
pub struct NewSubmission {
    pub title: String,
    pub comment: String,
    pub student: u64,
    pub workshop: u64,
    pub date: chrono::NaiveDateTime,
    pub locked: bool,
    pub reviewsdone: bool,
    pub error: bool,
}

#[derive(Insertable, Queryable, Clone)]
#[table_name = "submissionattachments"]
pub struct Submissionattachment {
    pub submission: u64,
    pub attachment: u64,
}

#[derive(Insertable, Queryable, Clone)]
#[table_name = "submissioncriteria"]
pub struct Submissioncriteria {
    pub submission: u64,
    pub criterion: u64,
}

/*
CREATE TABLE reviews
(
    id         SERIAL PRIMARY KEY,
    feedback   TEXT            NOT NULL,
    reviewer   BIGINT UNSIGNED,
    submission BIGINT UNSIGNED NOT NULL,
    workshop   BIGINT UNSIGNED NOT NULL,
    deadline   DATETIME        NOT NULL,
    done       BOOL            NOT NULL,
    locked     BOOL            NOT NULL,
    error      BOOl            NOT NULL,
    FOREIGN KEY (reviewer) REFERENCES users (id) ON DELETE SET NULL,
    FOREIGN KEY (submission) REFERENCES submissions (id) ON DELETE CASCADE,
    FOREIGN KEY (workshop) REFERENCES workshops (id) ON DELETE CASCADE
);

CREATE TABLE reviewpoints
(
    review    BIGINT UNSIGNED NOT NULL,
    criterion BIGINT UNSIGNED NOT NULL,
    points    DOUBLE,
    PRIMARY KEY (review, criterion),
    FOREIGN KEY (review) REFERENCES submissions (id) ON DELETE CASCADE,
    FOREIGN KEY (criterion) REFERENCES criterion (id) ON DELETE CASCADE
);
 */

#[derive(Queryable, AsChangeset, Clone)]
pub struct Review {
    pub id: u64,
    pub feedback: String,
    pub reviewer: Option<u64>,
    pub submission: u64,
    pub workshop: u64,
    pub deadline: chrono::NaiveDateTime,
    pub done: bool,
    pub locked: bool,
    pub error: bool,
}

#[derive(Insertable, Queryable, Clone)]
#[table_name = "reviews"]
pub struct NewReview {
    pub feedback: String,
    pub reviewer: Option<u64>,
    pub submission: u64,
    pub workshop: u64,
    pub deadline: chrono::NaiveDateTime,
    pub done: bool,
    pub locked: bool,
    pub error: bool,
}

#[derive(Insertable, Queryable, Clone)]
#[table_name = "reviewpoints"]
pub struct ReviewPoints {
    pub review: u64,
    pub criterion: u64,
    pub points: f64,
}
