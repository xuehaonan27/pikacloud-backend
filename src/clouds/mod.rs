use async_trait::async_trait;

use crate::models::CloudCreateInfo;

pub mod openstack;

#[async_trait]
pub trait BaseCloudProvider: Send {
    fn name(&self) -> &'static str;

    async fn get_admin_token(&mut self) -> Result<String, CloudError>;

    async fn create_user(&mut self, username: String) -> Result<CloudCreateInfo, CloudError>;

    async fn delete_user(&mut self, provider_id: String) -> Result<(), CloudError>;

    async fn is_user_exist(&mut self, provider_id: String) -> Result<bool, CloudError>;

    async fn get_user_token(
        &mut self,
        provider_id: String,
        provider_pass: String,
    ) -> Result<String, CloudError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CloudError {
    #[error("Sending request: {0}")]
    SendRequest(String),
    #[error("NotFound: {0}")]
    NotFound(String),
    #[error("Provider: {0}")]
    Provider(#[from] reqwest::Error),
}
