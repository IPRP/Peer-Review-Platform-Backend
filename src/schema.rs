table! {
    users (id) {
        id -> Unsigned<Bigint>,
        username -> Varchar,
        firstname -> Varchar,
        lastname -> Varchar,
        password -> Varchar,
        role -> Varchar,
        unit -> Nullable<Varchar>,
    }
}
