// @generated automatically by Diesel CLI.

diesel::table! {
    messages (id) {
        id -> Integer,
        user_id -> Integer,
        #[sql_name = "type"]
        type_ -> SmallInt,
        rx_msg_id -> BigInt,
        rx_msg -> Text,
        tx_msg_id -> BigInt,
    }
}

diesel::table! {
    notes (id) {
        id -> Integer,
        user_id -> Integer,
        key -> Text,
        value -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        telegram_id -> BigInt,
        topic -> BigInt,
        info_message -> Nullable<BigInt>,
        first_name -> Nullable<Text>,
        last_name -> Nullable<Text>,
        lang_code -> Nullable<Text>,
    }
}

diesel::joinable!(messages -> users (user_id));
diesel::joinable!(notes -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    messages,
    notes,
    users,
);
