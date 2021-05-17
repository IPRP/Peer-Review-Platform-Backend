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
        end -> Date,
        anonymous -> Bool,
    }
}

joinable!(criteria -> criterion (criterion));
joinable!(criteria -> workshops (workshop));
joinable!(workshoplist -> users (user));
joinable!(workshoplist -> workshops (workshop));

allow_tables_to_appear_in_same_query!(criteria, criterion, users, workshoplist, workshops,);
