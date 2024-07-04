use std::io::Write;

use chrono::NaiveDateTime;

use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::*;

use serde::{Deserialize, Serialize};

use crate::schema::sql_types::CloudProvider as CloudProviderType;
use crate::schema::sql_types::LoginProvider as LoginProviderType;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq)]
#[diesel(sql_type = LoginProviderType)]
pub enum LoginProvider {
    IAAA,
    PASSWORD,
}

impl ToSql<LoginProviderType, Pg> for LoginProvider {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        match *self {
            LoginProvider::IAAA => out.write_all(b"IAAA")?,
            LoginProvider::PASSWORD => out.write_all(b"PASSWORD")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<LoginProviderType, Pg> for LoginProvider {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"IAAA" => Ok(LoginProvider::IAAA),
            b"PASSWORD" => Ok(LoginProvider::PASSWORD),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq)]
#[diesel(sql_type = CloudProviderType)]
pub enum CloudProvider {
    OPENSTACK,
    PIKACLOUD,
}

impl ToSql<CloudProviderType, Pg> for CloudProvider {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            CloudProvider::OPENSTACK => out.write_all(b"OPENSTACK")?,
            CloudProvider::PIKACLOUD => out.write_all(b"PIKACLOUD")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<CloudProviderType, Pg> for CloudProvider {
    fn from_sql(bytes: <Pg as backend::Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"OPENSTACK" => Ok(CloudProvider::OPENSTACK),
            b"PIKACLOUD" => Ok(CloudProvider::PIKACLOUD),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, PartialEq, Insertable, Queryable, Identifiable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::User)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: String,
    pub username: String,
    pub loginProvider: LoginProvider,
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub createdAt: NaiveDateTime,
    pub updatedAt: NaiveDateTime,
}

// #[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
// pub struct NewUser {
//     pub username: String,

//     #[serde(rename = "loginProvider")]
//     pub login_provider: LoginProvider,

//     pub password: Option<String>,
// }

#[derive(Debug, PartialEq, Insertable, Queryable, Identifiable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::Role)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Role {
    pub id: String,
    pub name: String,
    pub createdAt: NaiveDateTime,
    pub updatedAt: NaiveDateTime,
}

#[derive(Debug, PartialEq, Insertable, Queryable, Identifiable, Associations, AsChangeset)]
#[belongs_to(User, foreign_key = "userId")]
#[belongs_to(Role, foreign_key = "roleId")]
#[diesel(table_name = crate::schema::UserRole)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserRole {
    pub id: String,
    pub userId: String,
    pub roleId: String,
    pub createdAt: NaiveDateTime,
    pub updatedAt: NaiveDateTime,
}

#[derive(Debug, PartialEq, Insertable, Queryable, Identifiable, Associations, AsChangeset)]
#[belongs_to(User, foreign_key = "userId")]
#[diesel(table_name = crate::schema::CloudUser)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CloudUser {
    pub id: String,
    pub userId: String,
    pub cloudProvider: CloudProvider,
    pub cloudUsername: String,
    pub cloudPassword: String,
    pub createdAt: NaiveDateTime,
    pub updatedAt: NaiveDateTime,
}

// #[derive(Serialize, Deserialize, Debug, sqlx::FromRow, sqlx::Type)]
// pub struct NewUserRole {
//     #[serde(rename = "userId")]
//     pub user_id: String,

//     #[serde(rename = "roleId")]
//     pub role_id: String,
// }

// #[derive(Serialize, Deserialize, Debug, sqlx::Type)]
// pub enum CloudProvider {
//     OPENSTACK,
//     PIKACLOUD,
// }

// #[derive(Serialize, Deserialize, Debug, sqlx::FromRow, sqlx::Type)]
// pub struct CloudUser {
//     pub id: String,

//     #[serde(rename = "userId")]
//     pub user_id: String,

//     #[serde(rename = "cloudProvider")]
//     pub cloud_provider: CloudProvider,

//     #[serde(rename = "cloudPassword")]
//     pub cloud_username: String,

//     #[serde(rename = "cloudPassword")]
//     pub cloud_password: String,

//     #[serde(rename = "createdAt")]
//     pub created_at: DateTime<Utc>,

//     #[serde(rename = "updatedAt")]
//     pub updated_at: DateTime<Utc>,
// }

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
