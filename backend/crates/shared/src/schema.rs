// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "statement_status"))]
    pub struct StatementStatus;
}

diesel::table! {
    categories (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 50]
        key -> Nullable<Varchar>,
        #[max_length = 50]
        icon -> Nullable<Varchar>,
        #[max_length = 7]
        color -> Nullable<Varchar>,
        is_system -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    refresh_tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        token_hash -> Varchar,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
        revoked_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::StatementStatus;

    statements (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 50]
        bank -> Varchar,
        #[max_length = 255]
        filename -> Varchar,
        #[max_length = 500]
        file_path -> Nullable<Varchar>,
        file_size -> Int8,
        status -> StatementStatus,
        transaction_count -> Nullable<Int4>,
        error_message -> Nullable<Text>,
        created_at -> Timestamptz,
        processed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    transactions (id, date) {
        id -> Uuid,
        user_id -> Uuid,
        statement_id -> Nullable<Uuid>,
        category_id -> Nullable<Uuid>,
        date -> Date,
        description -> Text,
        amount -> Numeric,
        balance -> Nullable<Numeric>,
        #[max_length = 100]
        reference -> Nullable<Varchar>,
        is_income -> Bool,
        is_user_categorized -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    transactions_2023 (id, date) {
        id -> Uuid,
        user_id -> Uuid,
        statement_id -> Nullable<Uuid>,
        category_id -> Nullable<Uuid>,
        date -> Date,
        description -> Text,
        amount -> Numeric,
        balance -> Nullable<Numeric>,
        #[max_length = 100]
        reference -> Nullable<Varchar>,
        is_income -> Bool,
        is_user_categorized -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    transactions_2024 (id, date) {
        id -> Uuid,
        user_id -> Uuid,
        statement_id -> Nullable<Uuid>,
        category_id -> Nullable<Uuid>,
        date -> Date,
        description -> Text,
        amount -> Numeric,
        balance -> Nullable<Numeric>,
        #[max_length = 100]
        reference -> Nullable<Varchar>,
        is_income -> Bool,
        is_user_categorized -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    transactions_2025 (id, date) {
        id -> Uuid,
        user_id -> Uuid,
        statement_id -> Nullable<Uuid>,
        category_id -> Nullable<Uuid>,
        date -> Date,
        description -> Text,
        amount -> Numeric,
        balance -> Nullable<Numeric>,
        #[max_length = 100]
        reference -> Nullable<Varchar>,
        is_income -> Bool,
        is_user_categorized -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    transactions_2026 (id, date) {
        id -> Uuid,
        user_id -> Uuid,
        statement_id -> Nullable<Uuid>,
        category_id -> Nullable<Uuid>,
        date -> Date,
        description -> Text,
        amount -> Numeric,
        balance -> Nullable<Numeric>,
        #[max_length = 100]
        reference -> Nullable<Varchar>,
        is_income -> Bool,
        is_user_categorized -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    transactions_2027 (id, date) {
        id -> Uuid,
        user_id -> Uuid,
        statement_id -> Nullable<Uuid>,
        category_id -> Nullable<Uuid>,
        date -> Date,
        description -> Text,
        amount -> Numeric,
        balance -> Nullable<Numeric>,
        #[max_length = 100]
        reference -> Nullable<Varchar>,
        is_income -> Bool,
        is_user_categorized -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(categories -> users (user_id));
diesel::joinable!(refresh_tokens -> users (user_id));
diesel::joinable!(statements -> users (user_id));
diesel::joinable!(transactions -> categories (category_id));
diesel::joinable!(transactions -> statements (statement_id));
diesel::joinable!(transactions -> users (user_id));
diesel::joinable!(transactions_2023 -> categories (category_id));
diesel::joinable!(transactions_2023 -> statements (statement_id));
diesel::joinable!(transactions_2023 -> users (user_id));
diesel::joinable!(transactions_2024 -> categories (category_id));
diesel::joinable!(transactions_2024 -> statements (statement_id));
diesel::joinable!(transactions_2024 -> users (user_id));
diesel::joinable!(transactions_2025 -> categories (category_id));
diesel::joinable!(transactions_2025 -> statements (statement_id));
diesel::joinable!(transactions_2025 -> users (user_id));
diesel::joinable!(transactions_2026 -> categories (category_id));
diesel::joinable!(transactions_2026 -> statements (statement_id));
diesel::joinable!(transactions_2026 -> users (user_id));
diesel::joinable!(transactions_2027 -> categories (category_id));
diesel::joinable!(transactions_2027 -> statements (statement_id));
diesel::joinable!(transactions_2027 -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    categories,
    refresh_tokens,
    statements,
    transactions,
    transactions_2023,
    transactions_2024,
    transactions_2025,
    transactions_2026,
    transactions_2027,
    users,
);
