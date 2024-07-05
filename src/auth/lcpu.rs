use std::env;

use async_trait::async_trait;
use serde::Deserialize;

use crate::auth::{get_result_from_resp, iaaa::IAAAValidateResponse};
use crate::{db::DBClient, utils::load_env_panic};

use super::{AuthError, BaseAuthProvider};

/// LCPU authentication provider
pub struct LcpuAuthProvider {
    client: DBClient,
    req_client: reqwest::Client,
    app_id: String,
    app_key: String,
    app_root: String,
    enable_mfa: bool,
}

#[async_trait]
impl BaseAuthProvider for LcpuAuthProvider {
    fn new(client: DBClient) -> Self
    where
        Self: Sized,
    {
        let app_id = load_env_panic("LCPU_APP_ID");
        let app_key = load_env_panic("LCPU_APP_KEY");
        let app_root = load_env_panic("LCPU_APP_ROOT");
        let enable_mfa = env::var("PIKA_ENABLE_MFA")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        Self {
            client,
            req_client: reqwest::Client::new(),
            app_id,
            app_key,
            app_root,
            enable_mfa,
        }
    }

    fn enable_mfa(&self) -> bool {
        self.enable_mfa
    }

    fn name(&self) -> &str {
        "lcpu"
    }

    async fn login(
        &mut self,
        payload: serde_json::Value,
        ip_address: Option<String>,
    ) -> Result<(String, Vec<String>), AuthError> {
        let conn = &mut self.client.get_conn()?;

        #[derive(Deserialize)]
        struct LoginPayLoad {
            token: String,
        }

        let LoginPayLoad { token } = serde_json::from_value(payload)
            .map_err(|_| AuthError::BadRequest("Token is required".into()))?;
        let ip_address = ip_address.ok_or(AuthError::Unauthorized("No ip address".into()))?;

        let f_str = format!(
            "appId={}&remoteAddr={}&token={}",
            self.app_id, ip_address, token
        );
        let msg_abs = format!("{:x}", md5::compute(&(f_str + &self.app_key)));

        let url = format!(
            "{}/api/oauth/iaaa_compat/svc/token/validate.do",
            self.app_root
        );
        let mut params = std::collections::HashMap::new();
        params.insert("remoteAddr", ip_address);
        params.insert("appId", self.app_id.clone());
        params.insert("token", token);
        params.insert("msgAbs", msg_abs);

        let res = self
            .req_client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| AuthError::Unauthorized(e.to_string()))?;

        let resp: IAAAValidateResponse = if res.status().is_success() {
            res.json()
                .await
                .map_err(|e| AuthError::Unauthorized(e.to_string()))?
        } else {
            return Err(AuthError::Unauthorized("Fail to send request".into()));
        };

        get_result_from_resp(conn, resp).await
    }

    async fn register(
        &mut self,
        _payload: serde_json::Value,
    ) -> Result<(String, Vec<String>), AuthError> {
        unreachable!("LCPU should not call register!")
    }
}
