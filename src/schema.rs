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
