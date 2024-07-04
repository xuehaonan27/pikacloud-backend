// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "CloudProvider"))]
    pub struct CloudProvider;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "LoginProvider"))]
    pub struct LoginProvider;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CloudProvider;

    CloudUser (id) {
        id -> Text,
        userId -> Text,
        cloudProvider -> CloudProvider,
        cloudUsername -> Text,
        cloudPassword -> Text,
        createdAt -> Timestamp,
        updatedAt -> Timestamp,
    }
}

diesel::table! {
    Role (id) {
        id -> Text,
        name -> Text,
        createdAt -> Timestamp,
        updatedAt -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LoginProvider;

    User (id) {
        id -> Text,
        username -> Text,
        loginProvider -> LoginProvider,
        name -> Nullable<Text>,
        email -> Nullable<Text>,
        password -> Nullable<Text>,
        createdAt -> Timestamp,
        updatedAt -> Timestamp,
    }
}

diesel::table! {
    UserRole (id) {
        id -> Text,
        userId -> Text,
        roleId -> Text,
        createdAt -> Timestamp,
        updatedAt -> Timestamp,
    }
}

diesel::joinable!(CloudUser -> User (userId));
diesel::joinable!(UserRole -> Role (roleId));
diesel::joinable!(UserRole -> User (userId));

diesel::allow_tables_to_appear_in_same_query!(
    CloudUser,
    Role,
    User,
    UserRole,
);
