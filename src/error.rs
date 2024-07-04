pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Authentication: {:?}", .0)]
    Auth(#[from] crate::auth::AuthError),
    #[error("DataBase {:?}", .0)]
    DataBase(#[from] crate::db::DBError),
}
