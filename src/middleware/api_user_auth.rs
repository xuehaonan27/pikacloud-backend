use std::future::{ready, Ready};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::AUTHORIZATION,
    Error, HttpResponse,
};
use futures_util::{future::LocalBoxFuture, FutureExt};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use crate::{models::UserJwtInfo, utils::load_env_panic};
pub struct ApiUserAuth;

impl<S, B> Transform<S, ServiceRequest> for ApiUserAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiUserAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiUserAuthMiddleware { service }))
    }
}

pub struct ApiUserAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ApiUserAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Handle before request
        println!("Hi from start. You requested: {}", req.path());
        let requested_path = req.uri().path();
        if requested_path.starts_with("/api/auth") {
            let http_res = HttpResponse::Unauthorized().finish();
            let (http_req, _) = req.into_parts();
            let res = ServiceResponse::new(http_req, http_res);
            return (async move { Ok(res.map_into_right_body()) }).boxed_local();
        }

        let auth_header = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok());
        if let Some(auth_header) = auth_header {
            let token = if auth_header.starts_with("Token ") {
                auth_header.replacen("Token ", "Bearer ", 1)
            } else {
                auth_header.to_string()
            };

            if token.starts_with("Bearer ") {
                let token = token.replacen("Bearer ", "", 1);
                // Verify token and get user info
                let secret = load_env_panic("JWT_SECRET");
                let user_info = verify_token(&token, &secret);

                if let Ok(user_info) = user_info {
                    // Search for admin role
                    let admin_role_found = user_info.roles.iter().find(|r| *r == "admin").is_some();

                    // Only admin role can access /api/admin, use route prefix to separate
                    if !((requested_path.starts_with("/api/admin")
                        || requested_path.starts_with("/admin"))
                        && !admin_role_found)
                    {
                        let fut = self.service.call(req);
                        return Box::pin(async move {
                            // Process response
                            let res = fut.await?;

                            // Handle after response
                            // println!("Hi from response");
                            Ok(res.map_into_left_body())
                        });
                    }
                }
            }
        }

        // If fail to call the service, then the control flow reaches here
        let http_res = HttpResponse::Unauthorized().finish();
        let (http_req, _) = req.into_parts();
        let res = ServiceResponse::new(http_req, http_res);
        return (async move { Ok(res.map_into_right_body()) }).boxed_local();
    }
}

fn verify_token(token: &str, secret: &str) -> jsonwebtoken::errors::Result<UserJwtInfo> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )?;
    Ok(token_data.claims)
}
