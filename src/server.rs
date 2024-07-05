use std::{io, sync::Arc};

use actix_web::{middleware, web, App, HttpServer};
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{
    auth::{iaaa::IaaaAuthProvider, password::PasswordAuthProvider, BaseAuthProvider},
    cache::RedisClient,
    clouds::{openstack::OpenStackCloudProvider, BaseCloudProvider},
    db::DBClient,
    middleware::api_user_auth::ApiUserAuth,
    routes::api_routes,
    utils::{load_env_optional, load_env_panic},
};

#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<Mutex<RedisClient>>,
    pub db: Arc<Mutex<DBClient>>,
    pub auth_providers: Arc<Mutex<Vec<Box<dyn BaseAuthProvider>>>>,
    pub cloud_providers: Arc<Mutex<Vec<Box<dyn BaseCloudProvider>>>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    trust_proxy: Option<String>,
    redis_url: String,
    database_url: String,
    cloud_providers: Vec<String>,
    auth_providers: Vec<String>,
}

pub async fn server() -> io::Result<()> {
    env_logger::init();
    let config = load_config();
    if config.trust_proxy.is_some() {
        log::warn!("Trust proxy is enabled");
    }
    let redis_client = RedisClient::new(&config.redis_url).await.unwrap();
    let db_client = DBClient::connect(&config.database_url).unwrap();

    let auth_providers = load_auth_providers(&config.auth_providers, db_client.clone()).await;
    let cloud_providers = load_cloud_providers(&config.cloud_providers, redis_client.clone()).await;

    let cache = Arc::new(Mutex::new(redis_client));
    let db = Arc::new(Mutex::new(db_client.clone()));
    let auth_providers = Arc::new(Mutex::new(auth_providers));
    let cloud_providers = Arc::new(Mutex::new(cloud_providers));

    let state = AppState {
        cache,
        db,
        auth_providers,
        cloud_providers,
    };

    log::info!("Server is ready");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(middleware::Logger::default()) // Attach a logger
            .wrap(ApiUserAuth) // Filter non-admin access to "/api/admin" or "/admin"
            .configure(configure_services)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn load_config() -> Config {
    let auth_providers: Vec<String> = load_env_panic("AUTH_PROVIDERS")
        .split(',')
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let cloud_providers: Vec<String> = load_env_panic("CLOUD_PROVIDER")
        .split(',')
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let database_url = load_env_panic("DATABASE_URL");
    let redis_url = load_env_panic("REDIS_URL");
    let trust_proxy = load_env_optional("TRUST_PROXY");
    Config {
        trust_proxy,
        redis_url,
        database_url,
        cloud_providers,
        auth_providers,
    }
}

async fn load_auth_providers(
    auth_providers: &[String],
    db: DBClient,
) -> Vec<Box<dyn BaseAuthProvider>> {
    let mut providers: Vec<Box<dyn BaseAuthProvider>> = vec![
        Box::new(PasswordAuthProvider::new(db.clone())),
        Box::new(IaaaAuthProvider::new(db.clone())),
        // Box::new(LcpuAuthProvider::new(db)),
    ];
    providers.retain(|provider| auth_providers.contains(&provider.name().to_string()));
    providers
}

async fn load_cloud_providers(
    cloud_providers: &[String],
    cache: RedisClient,
) -> Vec<Box<dyn BaseCloudProvider>> {
    let mut providers: Vec<Box<dyn BaseCloudProvider>> =
        vec![Box::new(OpenStackCloudProvider::new(cache.clone()))];
    providers.retain(|provider| cloud_providers.contains(&provider.name().to_string()));
    providers
}

fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api").configure(api_routes));
}
