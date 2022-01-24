use crate::models::*;

// Reviews
/// Simplified representation of review points.
#[derive(Serialize)]
pub struct SimpleReviewPoints {
    pub weight: f64,
    pub kind: Kind,
    pub points: f64,
}

/// Detailed representation of a review.
#[derive(Serialize)]
pub struct FullReview {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firstname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastname: Option<String>,
    pub feedback: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "notSubmitted")]
    pub not_submitted: Option<bool>,
    pub points: Vec<FullReviewPoints>,
}

/// Detailed representation of review points.
#[derive(Debug, Serialize)]
pub struct FullReviewPoints {
    #[serde(rename = "id")]
    pub criterion_id: u64,
    pub title: String,
    pub content: String,
    pub weight: f64,
    #[serde(rename = "type")]
    pub kind: Kind,
    pub points: f64,
}

/// Representation of a missing review
#[derive(Serialize)]
pub struct MissingReview {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firstname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastname: Option<String>,
}

/// Workshop representation of a review.
#[derive(Serialize)]
pub struct WorkshopReview {
    pub id: u64,
    pub done: bool,
    pub deadline: chrono::NaiveDateTime,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firstname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastname: Option<String>,
}

// Submissions
/// Representation of a submission for owner.
#[derive(Serialize)]
pub struct OwnSubmission {
    pub title: String,
    pub comment: String,
    pub attachments: Vec<SimpleAttachment>,
    pub locked: bool,
    pub date: chrono::NaiveDateTime,
    #[serde(rename(serialize = "reviewsDone"))]
    pub reviews_done: bool,
    #[serde(rename(serialize = "noReviews"))]
    pub no_reviews: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<f64>,
    #[serde(rename(serialize = "maxPoints"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_points: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firstname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastname: Option<String>,
    pub reviews: Vec<FullReview>,
    #[serde(rename(serialize = "missingReviews"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_reviews: Option<Vec<MissingReview>>,
}

/// Representation of a submission for other students like reviewers.
#[derive(Serialize)]
pub struct OtherSubmission {
    pub title: String,
    pub comment: String,
    pub attachments: Vec<SimpleAttachment>,
    pub criteria: Vec<Criterion>,
}

/// Workshop representation of a submission.
#[derive(Serialize)]
pub struct WorkshopSubmission {
    pub id: u64,
    pub title: String,
    pub date: chrono::NaiveDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    #[serde(rename(serialize = "studentid"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub student_id: Option<u64>,
    #[serde(rename(serialize = "reviewsDone"))]
    pub reviews_done: bool,
    #[serde(rename(serialize = "noReviews"))]
    pub no_reviews: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<f64>,
    #[serde(rename(serialize = "maxPoints"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_points: Option<f64>,
}

// Todos
/// Representation of a review for TODOs
#[derive(Serialize)]
pub struct TodoReview {
    pub id: u64,
    pub done: bool,
    pub deadline: chrono::NaiveDateTime,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firstname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastname: Option<String>,
    pub submission: u64,
    #[serde(rename(serialize = "workshopName"))]
    pub workshop_name: String,
}

/// Representation of a submission for TODOs
#[derive(Serialize)]
pub struct TodoSubmission {
    pub id: u64,
    #[serde(rename(serialize = "workshopName"))]
    pub workshop_name: String,
}

/// Representation of a student T O D O.
#[derive(Serialize)]
pub struct Todo {
    pub reviews: Vec<TodoReview>,
    pub submissions: Vec<TodoSubmission>,
}

// Users
/// Simplified representation of an user.
#[derive(Serialize, Clone)]
pub struct SimpleUser {
    pub id: u64,
    pub firstname: String,
    pub lastname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "group")]
    pub unit: Option<String>,
}

// Workshops
/// Workshop representation of an user.
#[derive(Serialize)]
pub struct WorkshopUser {
    pub id: u64,
    pub firstname: String,
    pub lastname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submissions: Option<Vec<WorkshopSubmission>>,
}

/// Teacher representation of a workshop.
#[derive(Serialize)]
pub struct TeacherWorkshop {
    pub title: String,
    pub content: String,
    pub end: chrono::NaiveDateTime,
    #[serde(rename(serialize = "reviewTimespan"))]
    pub review_timespan: i64,
    pub anonymous: bool,
    pub students: Vec<WorkshopUser>,
    pub teachers: Vec<WorkshopUser>,
    pub criteria: Vec<Criterion>,
    pub attachments: Vec<SimpleAttachment>,
}

/// Student representation of a workshop.
#[derive(Serialize)]
pub struct StudentWorkshop {
    pub title: String,
    pub content: String,
    pub end: chrono::NaiveDateTime,
    pub anonymous: bool,
    pub students: Vec<WorkshopUser>,
    pub teachers: Vec<WorkshopUser>,
    pub submissions: Vec<WorkshopSubmission>,
    pub reviews: Vec<WorkshopReview>,
    pub attachments: Vec<SimpleAttachment>,
}
