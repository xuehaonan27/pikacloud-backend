use std::env;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{auth::AuthError, db::DBClient};

use super::{password::PasswordAuthProvider, AuthResult, BaseAuthProvider};

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct IAAAUserInfo {
    // example 'Tom'
    #[serde(rename = "name")]
    name: String,

    // example: 'Kaitong'
    #[serde(rename = "status")]
    status: String,

    // example: '2200088888'
    #[serde(rename = "identityId")]
    identity_id: String,

    // example: '00048'
    #[serde(rename = "deptId")]
    dept_id: String,

    // example: '信息科学技术学院'
    #[serde(rename = "dept")]
    dept: String,

    // example: '学生'
    #[serde(rename = "identityType")]
    identity_type: String,

    // example: '本专科学生'
    #[serde(rename = "detailType")]
    detail_type: String,

    // example: '在校'
    #[serde(rename = "identityStatus")]
    identity_status: String,

    // example: '燕园'
    #[serde(rename = "campus")]
    campus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct IAAAValidateResponse {
    #[serde(rename = "success")]
    success: bool,
    #[serde(rename = "errCode")]
    err_code: String,
    #[serde(rename = "errMsg")]
    err_msg: String,
    #[serde(rename = "userInfo")]
    user_info: IAAAUserInfo,
}

impl IAAAValidateResponse {
    pub fn is_success(&self) -> bool {
        self.success
    }
}

fn md5_hash(msg: &String) -> String {
    let digest = md5::compute(msg);
    format!("{:x}", digest)
}

const VALIDATE_ENDPOINT: &'static str = "https://iaaa.pku.edu.cn/iaaa/svc/token/validate.do";

pub async fn validate(
    remote_addr: String,
    app_id: String,
    app_key: String,
    token: String,
) -> IaaaResult<IAAAValidateResponse> {
    let payload = format!("appId={app_id}&remoteAddr={remote_addr}&token={token}");
    let sign = md5_hash(&(payload.clone() + &app_key));
    let url = format!("{VALIDATE_ENDPOINT}?{payload}&msgAbs={sign}");
    let data = reqwest::get(url)
        .await
        .map_err(|e| IaaaError::Get(e.to_string()))?
        .json::<IAAAValidateResponse>()
        .await
        .map_err(|e| IaaaError::Deserialize(e.to_string()))?;
    return Ok(data);
}

pub type IaaaResult<T> = std::result::Result<T, IaaaError>;

#[derive(Debug, Clone)]
pub enum IaaaError {
    Get(String),
    Serialize(String),
    Deserialize(String),
}

/// IAAA authentication provider
pub struct IaaaAuthProvider {
    client: DBClient,
    iaaa_id: String,
    iaaa_key: String,
    enable_mfa: bool,
}

#[async_trait]
impl BaseAuthProvider for IaaaAuthProvider {
    fn enable_mfa(&self) -> bool {
        self.enable_mfa
    }

    fn name(&self) -> &'static str {
        "iaaa"
    }

    /// Login with IAAA authentication
    async fn login(
        &mut self,
        payload: serde_json::Value,
    ) -> Result<(String, Vec<String>), AuthError> {
        #[derive(Deserialize)]
        struct LoginPayLoad {
            token: String,
        }

        let login_payload: LoginPayLoad = serde_json::from_value(payload)
            .map_err(|_| AuthError::BadRequest("Token is required".into()))?;

        todo!()
    }

    async fn register(
        &mut self,
        payload: serde_json::Value,
    ) -> Result<(String, Vec<String>), AuthError> {
        unreachable!("IAAA should not call register!")
    }
}

impl IaaaAuthProvider {
    pub fn new(client: DBClient) -> Self {
        let iaaa_id = env::var("IAAA_ID").expect("Must set IAAA_ID");
        let iaaa_key = env::var("IAAA_KEY").expect("Must set IAAA_KEY");

        let enable_mfa = env::var("PIKA_ENABLE_MFA")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        Self {
            iaaa_id,
            iaaa_key,
            client,
            enable_mfa,
        }
    }
}