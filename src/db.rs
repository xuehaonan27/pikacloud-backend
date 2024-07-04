use std::env;

use redis::{aio::MultiplexedConnection, AsyncCommands, RedisError};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::models::{NewUser, NewUserRole, Role, User, UserRole};

#[derive(Debug, thiserror::Error)]
pub enum DBError {
    #[error("Fail to connect database")]
    Connection(#[from] sqlx::Error),
    #[error("Fail to read database url from environment")]
    Url(#[from] std::env::VarError),
    #[error("Fail to get field `{0}`")]
    Field(String),
    #[error("Redis failure: {0}")]
    Redis(#[from] RedisError),
}

pub type DBResult<T> = std::result::Result<T, DBError>;

/// Database client. Since `PgPool` is clone-safe, `DBClient` is clone-safe as well.
#[derive(Debug, Clone)]
pub struct DBClient {
    conn: PgPool,
}

impl DBClient {
    pub async fn connect(database_url: &String) -> DBResult<Self> {
        let conn = PgPoolOptions::new().connect(database_url).await?;
        Ok(Self {
            conn,
        })
    }

    pub fn user(&mut self) -> UserTable {
        UserTable::new(&mut self.conn)
    }

    pub fn user_role(&mut self) -> UserRoleTable {
        UserRoleTable::new(&mut self.conn)
    }

    pub fn role(&mut self) -> RoleTable {
        RoleTable::new(&mut self.conn)
    }
}

pub struct UserTable<'a> {
    name: &'static str,
    conn: &'a PgPool,
}

impl<'a> UserTable<'a> {
    pub fn new(conn: &'a mut PgPool) -> Self {
        Self { name: "user", conn }
    }

    /// Find a unique value
    pub async fn find_unique(&mut self, by: &str, value: &String) -> DBResult<User> {
        let sql = r#"
            SELECT * FROM "User"
            WHERE $1 = $2
            LIMIT 1;
        "#;
        let result = sqlx::query_as::<_, User>(sql)
            .bind(by)
            .bind(value)
            .fetch_one(self.conn)
            .await?;
        Ok(result)
    }

    /// Create a new user
    pub async fn create(&mut self, data: NewUser) -> DBResult<User> {
        todo!()
    }
}

pub struct UserRoleTable<'a> {
    name: &'static str,
    conn: &'a mut PgPool,
}

impl<'a> UserRoleTable<'a> {
    pub fn new(conn: &'a mut PgPool) -> Self {
        Self {
            name: "userrole",
            conn,
        }
    }

    pub async fn find_many<S: AsRef<str>>(
        &mut self,
        by: S,
        value: &String,
    ) -> DBResult<Vec<UserRole>> {
        todo!()
    }

    pub async fn create(&mut self, new_user_role: NewUserRole) -> DBResult<UserRole> {
        todo!()
    }
}

pub struct RoleTable<'a> {
    name: &'static str,
    conn: &'a mut PgPool,
}

impl<'a> RoleTable<'a> {
    pub fn new(conn: &'a mut PgPool) -> Self {
        Self { name: "role", conn }
    }

    pub async fn find_first(&mut self, by: &str, value: &String) -> DBResult<Role> {
        todo!()
    }

    pub async fn find_many<S: AsRef<str>, T>(
        &mut self,
        by: S,
        value: Vec<T>,
    ) -> DBResult<Vec<Role>> {
        todo!()
    }
}

#[derive(Clone)]
pub struct RedisClient {
    conn: MultiplexedConnection,
}

impl RedisClient {
    pub async fn new(redis_url: &String) -> DBResult<Self> {
        let client = redis::Client::open(redis_url.as_str())?;
        let conn = client.get_multiplexed_tokio_connection().await?;
        Ok(Self { conn })
    }

    pub async fn get(&mut self, key: &str) -> Option<String> {
        self.conn.get(key).await.unwrap_or(None)
    }

    pub async fn set(&mut self, key: &str, value: &str, expiration: u64) -> DBResult<()> {
        let _: () = self.conn.set_ex(key, value, expiration).await?;
        Ok(())
    }
}
