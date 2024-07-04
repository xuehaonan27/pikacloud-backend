use async_trait::async_trait;
use serde::Deserialize;
use std::env;

use crate::{db::DBClient, models};

use super::{encrypt, AuthError, BaseAuthProvider};

pub struct PasswordAuthProvider {
    client: DBClient,
    enable_mfa: bool,
    allow_password_login: bool,
    allow_register: bool,
}

impl PasswordAuthProvider {
    pub fn new(client: DBClient) -> Self {
        let allow_password_login = env::var("PIKA_ALLOW_PASSWORD_LOGIN")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let allow_register = env::var("PIKA_ALLOW_PASSWORD_REGISTER")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let enable_mfa = env::var("PIKA_ENABLE_MFA")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        Self {
            client,
            enable_mfa,
            allow_password_login,
            allow_register,
        }
    }
}

#[async_trait]
impl BaseAuthProvider for PasswordAuthProvider {
    fn enable_mfa(&self) -> bool {
        self.enable_mfa
    }

    fn name(&self) -> &str {
        "password"
    }

    async fn login(
        &mut self,
        payload: serde_json::Value,
    ) -> Result<(String, Vec<String>), AuthError> {
        #[derive(Deserialize)]
        struct LoginPayload {
            username: String,
            password: String,
        }

        let login_payload: LoginPayload = serde_json::from_value(payload)
            .map_err(|_| AuthError::BadRequest("Invalid payload".into()))?;
        let LoginPayload { username, password } = login_payload;
        if username.is_empty() || password.is_empty() {
            return Err(AuthError::BadRequest(
                "Username and password are required".into(),
            ));
        }

        // Assume you have some ORM methods for finding user and roles
        let user = self.client.user().find_unique("username", &username).await;
        let user = match user {
            Err(e) => {
                return Err(AuthError::Unauthorized(format!(
                    "Invalid username or password: {e}"
                )))
            }
            Ok(user) => user,
        };

        // Crypt password and user's password, and compare them
        let is_match = encrypt::compare(&password, &user.password);
        // .map_err(|_| AuthError::InternalServerError("Bcrypt verification failed".into()))?;

        if !is_match {
            return Err(AuthError::Unauthorized(
                "Invalid username or password".into(),
            ));
        }

        // Find UserRoles by user id
        let roles = self.client.user_role().find_many("id", &user.id).await?;
        let roles_id = roles.into_iter().map(|r| r.role_id).collect();

        // For each UserRole's role id, get correspond Role's name
        let role_names = self.client.role().find_many("id", roles_id).await?;

        Ok((user.id, role_names.into_iter().map(|r| r.name).collect()))
    }

    async fn register(
        &mut self,
        payload: serde_json::Value,
    ) -> Result<(String, Vec<String>), AuthError> {
        #[derive(Deserialize)]
        struct RegisterPayload {
            username: String,
            password: String,
        }

        if !self.allow_register {
            return Err(AuthError::Forbidden("Register is disabled".into()));
        }

        let register_payload: RegisterPayload = serde_json::from_value(payload)
            .map_err(|_| AuthError::BadRequest("Username and password are required".into()))?;
        let RegisterPayload { username, password } = register_payload;

        if username.chars().all(|c| c.is_numeric()) {
            return Err(AuthError::BadRequest(
                "Username cannot be all numbers".into(),
            ));
        }

        let existing_user = self.client.user().find_unique("username", &username).await;
        if existing_user.is_ok() {
            return Err(AuthError::Conflict("User already exists".into()));
        }

        let hashed_password = encrypt::auth_hash(&password, 10);
        let new_user = self
            .client
            .user()
            .create(models::NewUser {
                username,
                password: Some(hashed_password),
                login_provider: models::LoginProvider::PASSWORD,
            })
            .await
            .map_err(|_| AuthError::InternalServerError("Failed to create user".into()))?;

        let default_role = self
            .client
            .role()
            .find_first("name", &"member".to_string())
            .await
            .map_err(|_| AuthError::InternalServerError("Default role not found".into()))?;

        self.client
            .user_role()
            .create(models::NewUserRole {
                user_id: new_user.id.clone(),
                role_id: default_role.id,
            })
            .await
            .map_err(|_| AuthError::InternalServerError("Failed to create new user role".into()))?;

        Ok((new_user.id, vec!["member".into()]))
    }
}
