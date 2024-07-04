//! Authentication routes

use actix_session::Session;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{models::UserJwtInfo, server::AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetProviders {
    providers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LoginRequest {
    provider: String,
    payload: serde_json::Value,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegisterRequest {
    provider: String,
    payload: serde_json::Value,
    // username: String,
    // password: String,
}

#[derive(Deserialize)]
struct OAuthQuery {
    code: String,
}

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::get().to(get_auth_providers_handler)))
        .service(web::resource("/login").route(web::post().to(login_handler)))
        .service(web::resource("/register").route(web::post().to(register_handler)))
        .service(
            web::resource("/callback/{provider}").route(web::get().to(oauth_callback_handler)),
        );
}

async fn get_auth_providers_handler(data: web::Data<AppState>) -> HttpResponse {
    let auth_providers = data.auth_providers.lock().await;
    let providers = auth_providers
        .iter()
        .map(|x| x.name().to_string())
        .collect::<Vec<String>>();
    HttpResponse::Ok().json(GetProviders { providers })
}

async fn login_handler(
    data: web::Data<AppState>,
    req: web::Json<LoginRequest>,
    session: Session,
) -> HttpResponse {
    let mut auth_providers = data.auth_providers.lock().await;
    let provider = req.provider.clone();

    if let Some(auth_provider) = auth_providers.iter_mut().find(|p| p.name() == provider) {
        match auth_provider.login(req.payload.clone()).await {
            Ok((user_id, roles)) => {
                let user_info = UserJwtInfo { id: user_id, roles };
                session.insert("user_info", &user_info).unwrap();
                session.insert("expiresIn", "1d").unwrap();
                HttpResponse::Ok().json(user_info)
            }
            Err(err) => HttpResponse::BadRequest().body(err.to_string()),
        }
    } else {
        HttpResponse::BadRequest().body("Invalid provider")
    }
}

async fn register_handler(
    data: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
    session: Session,
) -> HttpResponse {
    let mut auth_providers = data.auth_providers.lock().await;
    let provider = req.provider.clone();

    if let Some(auth_provider) = auth_providers.iter_mut().find(|p| p.name() == provider) {
        match auth_provider.register(req.payload.clone()).await {
            Ok((user_id, roles)) => {
                let user_info = UserJwtInfo { id: user_id, roles };
                session.insert("user_info", &user_info).unwrap();
                session.insert("expiresIn", "1d").unwrap();
                HttpResponse::Ok().json(user_info)
            }
            Err(err) => HttpResponse::BadRequest().body(err.to_string()),
        }
    } else {
        HttpResponse::BadRequest().body("Invalid provider")
    }
}

async fn oauth_callback_handler(
    data: web::Data<AppState>,
    provider: web::Path<String>,
    query: web::Query<OAuthQuery>,
    session: Session,
) -> HttpResponse {
    let mut auth_providers = data.auth_providers.lock().await;
    let provider_name = provider.into_inner();

    if let Some(auth_provider) = auth_providers
        .iter_mut()
        .find(|p| p.name() == provider_name)
    {
        match auth_provider
            .login(serde_json::json!({ "token": query.code.clone() }))
            .await
        {
            Ok((user_id, roles)) => {
                let user_info = UserJwtInfo { id: user_id, roles };
                session.insert("user_info", &user_info).unwrap();
                HttpResponse::Ok().json(user_info)
            }
            Err(err) => HttpResponse::BadRequest().body(err.to_string()),
        }
    } else {
        HttpResponse::BadRequest().body("Invalid provider")
    }
}
