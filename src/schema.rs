table! {
    users (id) {
        id -> Unsigned<Bigint>,
        username -> Varchar,
        password -> Varchar,
        role -> Varchar,
        unit -> Nullable<Varchar>,
    }
}
