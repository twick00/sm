table! {
    file_details (id) {
        id -> Nullable<Integer>,
        file_path -> Text,
        data -> Binary,
        timestamp -> Integer,
    }
}

table! {
    file_diffs (id) {
        id -> Nullable<Integer>,
        original_file_id -> Nullable<Integer>,
        file_path -> Text,
        change_event -> Text,
        data -> Binary,
        timestamp -> Integer,
    }
}

joinable!(file_diffs -> file_details (original_file_id));

allow_tables_to_appear_in_same_query!(
    file_details,
    file_diffs,
);
