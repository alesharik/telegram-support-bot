// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Integer,
        telegram_id -> BigInt,
        topic -> BigInt,
        info_message -> Nullable<BigInt>,
    }
}
