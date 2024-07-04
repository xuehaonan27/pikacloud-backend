use async_trait::async_trait;

use crate::db::DBError;

pub mod encrypt;
pub mod iaaa;
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
    fn enable_mfa(&self) -> bool;
    fn name(&self) -> &str;
    async fn login(
        &mut self,
        payload: serde_json::Value,
    ) -> Result<(String, Vec<String>), AuthError>;
    async fn register(
        &mut self,
        payload: serde_json::Value,
    ) -> Result<(String, Vec<String>), AuthError>;
}
