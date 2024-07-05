use async_trait::async_trait;
use iaaa::IAAAValidateResponse;
use r2d2::{ConnectionManager, PooledConnection};

use crate::{
    db::{DBClient, DBError},
    models, schema,
};
use diesel::*;

pub mod iaaa;
pub mod lcpu;
pub mod password;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Token is required")]
    Token,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] DBError),
}

pub type AuthResult<T> = std::result::Result<T, AuthError>;

#[async_trait]
pub trait BaseAuthProvider: Send {
    fn new(client: DBClient) -> Self
    where
        Self: Sized;
    fn enable_mfa(&self) -> bool;
    fn name(&self) -> &str;
    async fn login(
        &mut self,
        payload: serde_json::Value,
        ip_address: Option<String>,
    ) -> Result<(String, Vec<String>), AuthError>;
    async fn register(
        &mut self,
        payload: serde_json::Value,
    ) -> Result<(String, Vec<String>), AuthError>;
}

pub async fn get_result_from_resp(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    resp: IAAAValidateResponse,
) -> Result<(String, Vec<String>), AuthError> {
    let user: Option<models::User> = schema::User::dsl::User
        .filter(schema::User::username.eq(&resp.user_info.identity_id))
        .select(models::User::as_select())
        .first(conn)
        .ok();

    let (user_id, user_roles) = if let Some(user) = user {
        let user_id = user.id;
        let roles = schema::UserRole::dsl::UserRole
            .filter(schema::UserRole::userId.eq(&user_id))
            .select(models::UserRole::as_select())
            .load(conn)
            .map_err(|_| AuthError::Unauthorized("Invalid username or password".into()))?;
        let role_ids: Vec<String> = roles.into_iter().map(|r| r.roleId).collect();
        let role_names: Vec<models::Role> = schema::Role::dsl::Role
            .filter(schema::Role::id.eq_any(role_ids))
            .select(models::Role::as_select())
            .load(conn)
            .map_err(|_| AuthError::Unauthorized("Invalid username or password".into()))?;
        let user_roles: Vec<String> = role_names.into_iter().map(|r| r.name).collect();
        (user_id, user_roles)
    } else {
        // Create new user
        let new_user = models::IaaaNewUser {
            username: resp.user_info.identity_id,
            loginProvider: models::LoginProvider::IAAA,
            name: Some(resp.user_info.name),
        };
        let new_user: models::User = diesel::insert_into(schema::User::table)
            .values(&new_user)
            .returning(models::User::as_returning())
            .get_result(conn)
            .map_err(|_| AuthError::InternalServerError("Failed to create user".into()))?;
        let user_id = new_user.id;
        // Update user role to "member"
        let role = schema::Role::dsl::Role
            .filter(schema::Role::name.eq(&"member".to_string()))
            .select(models::Role::as_select())
            .first(conn);

        let role: models::Role = match role {
            Ok(role) => role,
            Err(_) => {
                // Create role "member"
                let new_role = models::IaaaNewRole {
                    name: "member".into(),
                };
                let role = diesel::insert_into(schema::Role::table)
                    .values(&new_role)
                    .returning(models::Role::as_returning())
                    .get_result(conn)
                    .map_err(|_| AuthError::InternalServerError("Failed to create role".into()))?;
                role
            }
        };

        let new_user_role = models::NewUserRole {
            userId: user_id.clone(),
            roleId: role.id,
        };

        diesel::insert_into(schema::UserRole::table)
            .values(&new_user_role)
            .returning(models::UserRole::as_returning())
            .get_result(conn)
            .map_err(|_| AuthError::InternalServerError("Failed to create user role".into()))?;

        let user_roles = vec!["member".to_string()];

        (user_id, user_roles)
    };

    Ok((user_id, user_roles))
}
