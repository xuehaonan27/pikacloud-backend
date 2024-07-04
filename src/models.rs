use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
pub enum LoginProvider {
    IAAA,
    PASSWORD,
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct User {
    pub id: String,

    pub username: String,

    #[serde(rename = "loginProvider")]
    pub login_provider: LoginProvider,

    pub name: Option<String>,

    pub email: Option<String>,

    pub password: Option<String>,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct NewUser {
    pub username: String,

    #[serde(rename = "loginProvider")]
    pub login_provider: LoginProvider,

    pub password: Option<String>,
}


#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]

pub struct Role {
    pub id: String,

    pub name: String,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow, sqlx::Type)]
pub struct UserRole {
    pub id: String,

    #[serde(rename = "userId")]
    pub user_id: String,

    #[serde(rename = "roleId")]
    pub role_id: String,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow, sqlx::Type)]
pub struct NewUserRole {
    #[serde(rename = "userId")]
    pub user_id: String,

    #[serde(rename = "roleId")]
    pub role_id: String,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
pub enum CloudProvider {
    OPENSTACK,
    PIKACLOUD,
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow, sqlx::Type)]
pub struct CloudUser {
    pub id: String,

    #[serde(rename = "userId")]
    pub user_id: String,

    #[serde(rename = "cloudProvider")]
    pub cloud_provider: CloudProvider,

    #[serde(rename = "cloudPassword")]
    pub cloud_username: String,

    #[serde(rename = "cloudPassword")]
    pub cloud_password: String,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}


#[derive(Serialize, Deserialize, Debug, sqlx::FromRow, sqlx::Type)]
pub struct CloudCreateInfo {
    #[serde(rename = "providerId")]
    pub provider_id: String,

    #[serde(rename = "providerPass")]
    pub provider_pass: String,
}

/// Send to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserJwtInfo {
    pub id: String,
    pub roles: Vec<String>,
}