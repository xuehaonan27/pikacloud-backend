use redis::{aio::MultiplexedConnection, AsyncCommands};

#[derive(Clone)]
pub struct RedisClient {
    conn: MultiplexedConnection,
}

impl RedisClient {
    pub async fn new(redis_url: &String) -> CacheResult<Self> {
        let client = redis::Client::open(redis_url.as_str())
            .map_err(|e| CacheError::Connection(e.to_string()))?;
        let conn = client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| CacheError::Connection(e.to_string()))?;
        Ok(Self { conn })
    }

    pub async fn get(&mut self, key: &str) -> Option<String> {
        self.conn.get(key).await.unwrap_or(None)
    }

    pub async fn set(&mut self, key: &str, value: &str, expiration: u64) -> CacheResult<()> {
        let _: () = self
            .conn
            .set_ex(key, value, expiration)
            .await
            .map_err(|e| CacheError::Set(e.to_string()))?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Fail to connect: {0}")]
    Connection(String),
    #[error("Fail to set key: {0}")]
    Set(String),
}

pub type CacheResult<T> = std::result::Result<T, CacheError>;
