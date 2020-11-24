use actix_web::{error, get, post, web, App, HttpResponse, HttpServer, Responder};
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
    #[error("Pool error:`{0}`")]
    PoolError(PoolError),
    #[error("Redis error:`{0}`")]
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
    println!("GET: /hello");
    HttpResponse::Ok().body("Hello Redis")
}

#[get("/get/{key}")]
async fn get(pool: web::Data<Pool>, web::Path(key) : web::Path<String>) -> Result<HttpResponse, Error> {
    println!("GET: /get");
    let mut conn = pool.get().await?;
    let value = cmd("GET").arg(&[key]).query_async::<String>(&mut conn).await?;
    Ok(HttpResponse::Ok().json(value))
}

#[get("/set/{key}/{value}")]
async fn set(pool: web::Data<Pool>, web::Path((key, value)) : web::Path<(String, String)>) -> Result<HttpResponse, Error> {
    println!("GET: /set");
    let mut conn = pool.get().await?;
    cmd("SET").arg(&[&key, &value]).execute_async(&mut conn).await?;
    Ok(HttpResponse::Ok().body(format!("set {}:{}", key, value)))
}

#[get("/lpush/{key}/{value}")]
async fn lpush(redis: web::Data<Pool>, web::Path((key, value)) : web::Path<(String, String)>) -> impl Responder {
    println!("GET: /lpush");
    let mut conn = redis.get().await.unwrap();
    cmd("LPUSH").arg(&[&key, &value]).execute_async(&mut conn).await.unwrap();
    HttpResponse::Ok().body(format!("lpush {}:{}", key, value))
}

#[get("/lrange/{key}")]
async fn lrange(pool: web::Data<Pool>, web::Path(key) : web::Path<String>) -> Result<HttpResponse, Error>  {
    println!("GET: /lrange");
    let mut conn = pool.get().await?;
    let values  = cmd("LRANGE").arg(&[&key, "0", "-1"]).query_async::<Vec<String>>(&mut conn).await?;
    Ok(HttpResponse::Ok().json(values))
}

#[get("/incr/{key}")]
async fn incr(redis: web::Data<Pool>, web::Path(key) : web::Path<String>) -> Result<HttpResponse, Error> {
    println!("GET: /incr");
    let mut conn = redis.get().await?;
    cmd("INCR").arg(&[&key]).execute_async(&mut conn).await.unwrap();
    Ok(HttpResponse::Ok().body("incr"))
}

#[post("/mget")]
async fn mget(redis: web::Data<Pool>, keys: web::Json<Vec<String>>) -> Result<HttpResponse, Error> {
    println!("POST: /mget keys:{:?}", keys);
    let mut conn = redis.get().await.unwrap();
    let values = cmd("MGET")
        .arg(&**keys)
        .query_async::<Vec<String>>(&mut conn)
        .await?;
        
     Ok(HttpResponse::Ok().json(values))
}

#[get("/delete/{key}")]
async fn delete(redis: web::Data<Pool>, web::Path(key) : web::Path<String>) -> Result<HttpResponse, Error> {
    println!("GET: /delete");
    let mut conn = redis.get().await?;
    cmd("DEL").arg(&[&key]).execute_async(&mut conn).await.unwrap();
    Ok(HttpResponse::Ok().body(format!("delete {}", key)))
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
            .service(lpush)
            .service(lrange)
            .service(incr)
            .service(mget)
            .service(delete)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}