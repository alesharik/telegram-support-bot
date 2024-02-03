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
    users (id) {
        id -> Integer,
        telegram_id -> BigInt,
        topic -> BigInt,
        info_message -> Nullable<BigInt>,
    }
}

diesel::joinable!(messages -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    messages,
    users,
);
