use actix_web::web;
use auth::auth_routes;

pub mod auth;

pub fn api_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").configure(auth_routes));
}
