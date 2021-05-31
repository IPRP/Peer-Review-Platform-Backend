table! {
    use diesel::sql_types::*;
    use crate::models::*;

    attachments (id) {
        id -> Unsigned<Bigint>,
        title -> Varchar,
        owner -> Nullable<Unsigned<Bigint>>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::*;

    criteria (workshop, criterion) {
        workshop -> Unsigned<Bigint>,
        criterion -> Unsigned<Bigint>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::*;

    criterion (id) {
        id -> Unsigned<Bigint>,
        title -> Varchar,
        content -> Text,
        weight -> Nullable<Double>,
        kind -> KindMapping,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::*;

    reviewpoints (review, criterion) {
        review -> Unsigned<Bigint>,
        criterion -> Unsigned<Bigint>,
        points -> Nullable<Double>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::*;

    reviews (id) {
        id -> Unsigned<Bigint>,
        feedback -> Text,
        reviewer -> Nullable<Unsigned<Bigint>>,
        submission -> Unsigned<Bigint>,
        workshop -> Unsigned<Bigint>,
        deadline -> Datetime,
        done -> Bool,
        locked -> Bool,
        error -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::*;

    submissionattachments (submission, attachment) {
        submission -> Unsigned<Bigint>,
        attachment -> Unsigned<Bigint>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::*;

    submissioncriteria (submission, criterion) {
        submission -> Unsigned<Bigint>,
        criterion -> Unsigned<Bigint>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::*;

    submissions (id) {
        id -> Unsigned<Bigint>,
        title -> Varchar,
        comment -> Text,
        student -> Nullable<Unsigned<Bigint>>,
        workshop -> Unsigned<Bigint>,
        date -> Datetime,
        locked -> Bool,
        reviewsdone -> Bool,
        error -> Bool,
        meanpoints -> Nullable<Double>,
        maxpoint -> Nullable<Double>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::*;

    users (id) {
        id -> Unsigned<Bigint>,
        username -> Varchar,
        firstname -> Varchar,
        lastname -> Varchar,
        password -> Varchar,
        role -> RoleMapping,
        unit -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::*;

    workshoplist (workshop, user) {
        workshop -> Unsigned<Bigint>,
        user -> Unsigned<Bigint>,
        role -> RoleMapping,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::*;

    workshops (id) {
        id -> Unsigned<Bigint>,
        title -> Varchar,
        content -> Text,
        end -> Datetime,
        anonymous -> Bool,
    }
}

joinable!(attachments -> users (owner));
joinable!(criteria -> criterion (criterion));
joinable!(criteria -> workshops (workshop));
joinable!(reviewpoints -> criterion (criterion));
joinable!(reviewpoints -> submissions (review));
joinable!(reviews -> submissions (submission));
joinable!(reviews -> users (reviewer));
joinable!(reviews -> workshops (workshop));
joinable!(submissionattachments -> attachments (attachment));
joinable!(submissionattachments -> submissions (submission));
joinable!(submissioncriteria -> criterion (criterion));
joinable!(submissioncriteria -> submissions (submission));
joinable!(submissions -> users (student));
joinable!(submissions -> workshops (workshop));
joinable!(workshoplist -> users (user));
joinable!(workshoplist -> workshops (workshop));

allow_tables_to_appear_in_same_query!(
    attachments,
    criteria,
    criterion,
    reviewpoints,
    reviews,
    submissionattachments,
    submissioncriteria,
    submissions,
    users,
    workshoplist,
    workshops,
);
