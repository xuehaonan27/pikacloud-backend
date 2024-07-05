use async_trait::async_trait;
use serde::Deserialize;
use std::env;

use crate::{
    db::DBClient,
    models::{self, NewUserRole, PasswordNewUser},
    schema,
};

use super::{AuthError, BaseAuthProvider};
use diesel::prelude::*;

pub struct PasswordAuthProvider {
    client: DBClient,
    enable_mfa: bool,
    allow_password_login: bool,
    allow_register: bool,
}

#[async_trait]
impl BaseAuthProvider for PasswordAuthProvider {
    fn new(client: DBClient) -> Self
    where
        Self: Sized,
    {
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

    fn enable_mfa(&self) -> bool {
        self.enable_mfa
    }

    fn name(&self) -> &str {
        "password"
    }

    async fn login(
        &mut self,
        payload: serde_json::Value,
        _ip_address: Option<String>,
    ) -> Result<(String, Vec<String>), AuthError> {
        // Get connection to database
        let mut conn = self.client.get_conn()?;

        #[derive(Deserialize)]
        struct LoginPayload {
            f_username: String,
            f_password: String,
        }

        let login_payload: LoginPayload = serde_json::from_value(payload)
            .map_err(|_| AuthError::BadRequest("Invalid payload".into()))?;
        let LoginPayload {
            f_username,
            f_password,
        } = login_payload;
        if f_username.is_empty() || f_password.is_empty() {
            return Err(AuthError::BadRequest(
                "Username and password are required".into(),
            ));
        }

        let user = schema::User::dsl::User
            .filter(schema::User::username.eq(f_username))
            .select(models::User::as_select())
            .first(&mut conn);
        let user = match user {
            Err(e) => {
                return Err(AuthError::Unauthorized(format!(
                    "Invalid username or password: {e}"
                )));
            }
            Ok(user) => user,
        };

        // Crypt password and user's password, and compare them
        let is_match = user.password.is_some()
            && bcrypt::verify(&f_password, &user.password.unwrap())
                .map_err(|_| AuthError::Unauthorized("Invalid username or password".into()))?;

        if !is_match {
            return Err(AuthError::Unauthorized(
                "Invalid username or password".into(),
            ));
        }

        // Find UserRoles by user id
        // let roles = self.client.user_role().find_many("id", &user.id).await?;
        let roles = schema::UserRole::dsl::UserRole
            .filter(schema::UserRole::id.eq(&user.id))
            .select(models::UserRole::as_select())
            .load(&mut conn);
        let roles = match roles {
            Err(_) => {
                return Err(AuthError::Unauthorized(
                    "Invalid username or password".into(),
                ))
            }
            Ok(roles) => roles,
        };

        let roles_id: Vec<String> = roles.into_iter().map(|r| r.roleId).collect();

        // For each UserRole's role id, get correspond Role's name
        // let role_names = self.client.role().find_many("id", roles_id).await?;
        let role_names = schema::Role::dsl::Role
            .filter(schema::Role::id.eq_any(roles_id))
            .select(models::Role::as_select())
            .load(&mut conn);
        let role_names = match role_names {
            Err(_) => {
                return Err(AuthError::Unauthorized(
                    "Invalid username or password".into(),
                ))
            }
            Ok(role_names) => role_names,
        };

        Ok((user.id, role_names.into_iter().map(|r| r.name).collect()))
    }

    async fn register(
        &mut self,
        payload: serde_json::Value,
    ) -> Result<(String, Vec<String>), AuthError> {
        let mut conn = self.client.get_conn()?;
        #[derive(Deserialize)]
        struct RegisterPayload {
            f_username: String,
            f_password: String,
        }

        if !self.allow_register {
            return Err(AuthError::Forbidden("Register is disabled".into()));
        }

        let register_payload: RegisterPayload = serde_json::from_value(payload)
            .map_err(|_| AuthError::BadRequest("Username and password are required".into()))?;
        let RegisterPayload {
            f_username,
            f_password,
        } = register_payload;

        if f_username.chars().all(|c| c.is_numeric()) {
            return Err(AuthError::BadRequest(
                "Username cannot be all numbers".into(),
            ));
        }

        // let existing_user = self.client.user().find_unique("username", &username).await;
        let existing_user = schema::User::dsl::User
            .filter(schema::User::username.eq(&f_username))
            .select(models::User::as_select())
            .first(&mut conn);
        if existing_user.is_ok() {
            return Err(AuthError::Conflict("User already exists".into()));
        }

        let hashed_password = bcrypt::hash(&f_password, 10)
            .map_err(|_| AuthError::Unauthorized("bcrypt error".into()))?;
        let new_user = PasswordNewUser {
            username: f_username,
            password: Some(hashed_password),
            loginProvider: models::LoginProvider::PASSWORD,
        };
        let new_user = diesel::insert_into(schema::User::table)
            .values(&new_user)
            .returning(models::User::as_returning())
            .get_result(&mut conn)
            .map_err(|_| AuthError::InternalServerError("Failed to create user".into()))?;

        let default_role = schema::Role::dsl::Role
            .filter(schema::Role::name.eq(&"member".to_string()))
            .select(models::Role::as_select())
            .first(&mut conn)
            .map_err(|_| AuthError::InternalServerError("Default role not found".into()))?;

        let new_user_role = NewUserRole {
            userId: new_user.id.clone(),
            roleId: default_role.id,
        };

        diesel::insert_into(schema::UserRole::table)
            .values(&new_user_role)
            .returning(models::UserRole::as_returning())
            .get_result(&mut conn)
            .map_err(|_| AuthError::InternalServerError("Failed to create new user role".into()))?;

        Ok((new_user.id, vec!["member".into()]))
    }
}
