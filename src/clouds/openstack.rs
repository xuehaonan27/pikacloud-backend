use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{cache::RedisClient, models::CloudCreateInfo, utils::load_env_panic};

use super::{BaseCloudProvider, CloudError};

#[derive(Serialize)]
struct AuthRequest {
    auth: Auth,
}

#[derive(Serialize)]
struct Auth {
    identity: Identity,
}

#[derive(Serialize)]
struct Identity {
    methods: Vec<String>,
    password: Password,
}

#[derive(Serialize)]
struct User {
    name: String,
    domain: Domain,
    password: String,
}

#[derive(Serialize)]
struct Domain {
    name: String,
}

#[derive(Deserialize)]
struct AuthResponse {
    expires_at: Option<String>,
}

#[derive(Deserialize)]
struct DomainsResponse {
    domains: Vec<DomainInfo>,
}

#[derive(Deserialize)]
struct RolesResponse {
    roles: Vec<RoleInfo>,
}

#[derive(Serialize)]
struct Password {
    user: User,
}

#[derive(Deserialize)]
struct DomainInfo {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct RoleInfo {
    pub id: String,
    pub name: String,
}

pub struct OpenStackCloudProvider {
    cache: RedisClient,
    client: reqwest::Client,
}

impl OpenStackCloudProvider {
    pub fn new(cache: RedisClient) -> Self {
        Self {
            cache,
            client: reqwest::Client::new(),
        }
    }

    // Get default domain id, store in redis
    async fn get_default_domain_id(&mut self) -> Result<String, CloudError> {
        // Get default domain id, store in redis
        if let Some(domain_id) = self.cache.get("openstack_default_domain_id").await {
            return Ok(domain_id);
        }
        let keystone = load_env_panic("OPENSTACK_KEYSTONE");
        let token = self.get_admin_token().await?;

        // Get domain id by name 'Default'
        let response = self
            .client
            .get(format!("{}/domains", keystone))
            .header("X-Auth-Token", token)
            .send()
            .await
            .map_err(|e| CloudError::SendRequest(e.to_string()))?
            .json::<DomainsResponse>()
            .await?;
        let default_domain = response
            .domains
            .iter()
            .find(|domain| domain.name == "Default")
            .ok_or(CloudError::NotFound("domain name".into()))?;

        // Cache for 1 day
        self.cache
            .set("openstack_default_domain_id", &default_domain.id, 86400)
            .await
            .map_err(|e| CloudError::SendRequest(e.to_string()))?;
        Ok(default_domain.id.clone())
    }

    async fn get_member_role_id(&mut self) -> Result<String, CloudError> {
        if let Some(member_role_id) = self.cache.get("openstack_member_role_id").await {
            return Ok(member_role_id);
        }
        let keystone = load_env_panic("OPENSTACK_KEYSTONE");
        let token = self.get_admin_token().await?;
        // Get domain id by name 'Default'
        let response = self
            .client
            .get(format!("{}/roles", keystone))
            .header("X-Auth-Token", token)
            .send()
            .await
            .map_err(|e| CloudError::SendRequest(e.to_string()))?
            .json::<RolesResponse>()
            .await?;
        let member_role = response
            .roles
            .iter()
            .find(|role| role.name == "member")
            .ok_or(CloudError::NotFound("member role".into()))?;
        self.cache
            .set("openstack_member_role_id", &member_role.id, 86400)
            .await
            .map_err(|e| CloudError::SendRequest(e.to_string()))?;
        Ok(member_role.id.clone())
    }
}

#[async_trait]
impl BaseCloudProvider for OpenStackCloudProvider {
    fn name(&self) -> &'static str {
        "openstack"
    }

    async fn get_admin_token(&mut self) -> Result<String, CloudError> {
        if let Some(token) = self.cache.get("openstack:admin-token").await {
            return Ok(token);
        }

        let keystone = load_env_panic("OPENSTACK_KEYSTONE");
        let username = load_env_panic("OPENSTACK_ADMIN_USERNAME");
        let password = load_env_panic("OPENSTACK_ADMIN_PASSWORD");

        let response = self
            .client
            .post(format!("{}/auth/tokens", keystone))
            .json(&AuthRequest {
                auth: Auth {
                    identity: Identity {
                        methods: vec!["password".to_string()],
                        password: Password {
                            user: User {
                                name: username,
                                domain: Domain {
                                    name: "Default".to_string(),
                                },
                                password,
                            },
                        },
                    },
                },
            })
            .send()
            .await
            .map_err(|e| CloudError::SendRequest(e.to_string()))?
            .error_for_status()?;

        let new_token = response
            .headers()
            .get("X-Subject-Token")
            .ok_or(CloudError::NotFound("OpenStack admin token".into()))?
            .to_str()
            .map_err(|_| CloudError::NotFound("OpenStack admin token".into()))?
            .to_string();
        let response_json: AuthResponse = response
            .json()
            .await
            .map_err(|_| CloudError::NotFound("OpenStack admin token".into()))?;
        let expires_at = response_json
            .expires_at
            .ok_or(CloudError::NotFound("OpenStack admin token".into()))?;
        let expires_at_seconds = chrono::DateTime::parse_from_rfc3339(&expires_at)
            .map_err(|_| CloudError::NotFound("OpenStack admin token".into()))?
            .timestamp()
            - 300;
        self.cache
            .set(
                "openstack:admin-token",
                &new_token,
                expires_at_seconds as u64,
            )
            .await
            .map_err(|e| CloudError::SendRequest(e.to_string()))?;
        Ok(new_token)
    }

    async fn get_user_token(
        &mut self,
        provider_id: String,
        provider_pass: String,
    ) -> Result<String, CloudError> {
        if let Some(token) = self
            .cache
            .get(&format!("openstack:user-token-{}", provider_id))
            .await
        {
            return Ok(token);
        }
        let keystone = load_env_panic("OPENSTACK_KEYSTONE");
        let response = self
            .client
            .post(format!("{}/auth/tokens", keystone))
            .json(&AuthRequest {
                auth: Auth {
                    identity: Identity {
                        methods: vec!["password".to_string()],
                        password: Password {
                            user: User {
                                name: provider_id.clone(),
                                domain: Domain {
                                    name: "Default".to_string(),
                                },
                                password: provider_pass,
                            },
                        },
                    },
                },
            })
            .send()
            .await
            .map_err(|e| CloudError::SendRequest(e.to_string()))?
            .error_for_status()?;
        let new_token = response
            .headers()
            .get("X-Subject-Token")
            .ok_or(CloudError::NotFound("OpenStack user token".into()))?
            .to_str()
            .map_err(|_| CloudError::NotFound("OpenStack user token".into()))?
            .to_string();
        let response_json: AuthResponse = response
            .json()
            .await
            .map_err(|_| CloudError::NotFound("OpenStack user token".into()))?;
        let expires_at = response_json
            .expires_at
            .ok_or(CloudError::NotFound("OpenStack user token".into()))?;
        let expires_at_seconds = chrono::DateTime::parse_from_rfc3339(&expires_at)
            .map_err(|_| CloudError::NotFound("OpenStack user token".into()))?
            .timestamp()
            - 300;
        self.cache
            .set(
                &format!("openstack:user-token-{provider_id}"),
                &new_token,
                expires_at_seconds as u64,
            )
            .await
            .map_err(|e| CloudError::SendRequest(e.to_string()))?;
        Ok(new_token)
    }

    async fn create_user(&mut self, username: String) -> Result<CloudCreateInfo, CloudError> {
        let admin_token = self.get_admin_token().await?;
        let keystone = load_env_panic("OPENSTACK_KEYSTONE");

        #[derive(Serialize)]
        struct Project {
            pub name: String,
            pub domain_id: String,
        }

        #[derive(Deserialize)]
        struct ProjectResponse {
            pub id: Option<String>,
        }

        let response = self
            .client
            .post(&format!("{}/projects", keystone))
            .header("X-Auth-Token", &admin_token)
            .json(&Project {
                name: username.clone(),
                domain_id: "default".to_string(),
            })
            .send()
            .await
            .map_err(|e| CloudError::SendRequest(e.to_string()))?
            .error_for_status()?;

        let project: ProjectResponse = response
            .json()
            .await
            .map_err(|e| CloudError::SendRequest(e.to_string()))?;

        let provider_pass = uuid::Uuid::new_v4().to_string();

        #[derive(Deserialize)]
        struct CreateUserResponse {
            pub id: Option<String>,
        }

        #[derive(Serialize)]
        struct CreateUser {
            name: String,
            password: String,
            default_project_id: Option<String>,
        }

        let response = self
            .client
            .post(&format!("{}/users", keystone))
            .header("X-Auth-Token", &admin_token)
            .json(&CreateUser {
                name: username.clone(),
                password: provider_pass.clone(),
                default_project_id: project.id,
            })
            .send()
            .await
            .map_err(|e| CloudError::SendRequest(e.to_string()))?
            .error_for_status()?;

        let user: CreateUserResponse = response
            .json()
            .await
            .map_err(|e| CloudError::SendRequest(e.to_string()))?;
        if user.id.is_none() {
            log::error!("Failed to create user {username}");
            return Err(CloudError::NotFound(format!(
                "Failed to create user {username}"
            )));
        }
        let domain_id = self.get_default_domain_id().await?;
        let member_role_id = self.get_member_role_id().await?;
        let _response = self
            .client
            .put(&format!(
                "{keystone}/domains/{domain_id}/users/{0}/roles/{member_role_id}",
                user.id.unwrap()
            ))
            .header("X-Auth-Token", &admin_token)
            .send()
            .await
            .map_err(|e| CloudError::SendRequest(e.to_string()))?
            .error_for_status()?;

        Ok(CloudCreateInfo {
            provider_id: username,
            provider_pass,
        })
    }

    async fn delete_user(&mut self, provider_id: String) -> Result<(), CloudError> {
        let admin_token = self.get_admin_token().await?;
        let keystone = load_env_panic("OPENSTACK_KEYSTONE");
        let _response = self
            .client
            .delete(&format!("{keystone}/users/{provider_id}"))
            .header("X-Auth-Token", &admin_token)
            .send()
            .await
            .map_err(|e| CloudError::SendRequest(e.to_string()))?
            .error_for_status()?;
        Ok(())
    }

    async fn is_user_exist(&mut self, provider_id: String) -> Result<bool, CloudError> {
        let admin_token = self.get_admin_token().await?;
        let keystone = load_env_panic("OPENSTACK_KEYSTONE");
        let response = self
            .client
            .get(&format!("{keystone}/users/{provider_id}"))
            .header("X-Auth-Token", &admin_token)
            .send()
            .await
            .map_err(|e| CloudError::SendRequest(e.to_string()))?;
        Ok(response.status().is_success())
    }
}
