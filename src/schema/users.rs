use diesel::table;

table! {
    users (id) {
        id -> Varchar,
        username -> Varchar,
        email -> Nullable<Varchar>,
        phone_number -> Nullable<Varchar>,
        password -> String,
        activated -> Bool,
    }
}
