use actix_web::{error, get, web, App, HttpResponse, HttpServer, Responder};
use deadpool_redis::{cmd, Pool, PoolError};
use deadpool_redis::redis::RedisError;
use dotenv::dotenv;
use serde::Deserialize;
use thiserror::*;

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default)]
    redis: deadpool_redis::Config
}

impl Config {
    pub fn from_env() -> Result<Self, ::config_crate::ConfigError> {
        let mut cfg = ::config_crate::Config::new();
        cfg.merge(::config_crate::Environment::new().separator("__"))?;
        cfg.try_into()
    }
}

#[derive(Error, Debug)]
enum Error {
    #[error("An internal pool error occured. Please try again later.")]
    PoolError(PoolError),
    #[error(transparent)]
    RedisError(RedisError),
}

impl From<PoolError> for Error {
    fn from(error: PoolError) -> Self {
        Self::PoolError(error)
    }
}

impl From<RedisError> for Error {
    fn from(error: RedisError) -> Self {
        Self::RedisError(error)
    }
}

impl error::ResponseError for Error {}


#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello Redis")
}

#[get("/get/{key}")]
async fn get(pool: web::Data<Pool>, web::Path(key) : web::Path<String>) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().await?;
    let value = cmd("GET").arg(&[key]).query_async::<String>(&mut conn).await?;
    Ok(HttpResponse::Ok().json(value))
}

#[get("/set/{key}/{value}")]
async fn set(pool: web::Data<Pool>, web::Path((key, value)) : web::Path<(String, String)>) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().await?;
    cmd("SET").arg(&[&key, &value]).execute_async(&mut conn).await?;
    Ok(HttpResponse::Ok().body(format!("{}:{}", key, value)))
}

#[get("/except")]
async fn except() -> impl Responder {
    HttpResponse::BadRequest().body("error")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let cfg = Config::from_env().unwrap();
    let pool = cfg.redis.create_pool().unwrap();

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(hello)
            .service(except)
            .service(get)
            .service(set)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}