// @generated automatically by Diesel CLI.

diesel::table! {
    users (telegram_id) {
        telegram_id -> BigInt,
        topic -> BigInt,
    }
}
