use actix_web::{get, delete, put, web, HttpResponse, Responder, Scope};
use redis::Commands;
use serde::Deserialize;

use crate::REDIS_CLIENT;

pub fn redis_scope() -> Scope {
    web::scope("/redis").configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(redis_keys)
        .service(redis_get_key)
        .service(redis_delete_key)
        .service(redis_clear)
        .service(redis_update_key);
}

#[get("/keys")]
async fn redis_keys() -> impl Responder {
    match RedisService::list_keys("*").await {
        Ok(keys) => HttpResponse::Ok().json(keys),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/get/{key}")]
async fn redis_get_key(key: web::Path<String>) -> impl Responder {
    match RedisService::get_key(&key).await {
        Ok(Some(value)) => HttpResponse::Ok().body(value),
        Ok(None) => HttpResponse::NotFound().body("Key not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[delete("/delete/{key}")]
async fn redis_delete_key(key: web::Path<String>) -> impl Responder {
    match RedisService::delete_key(&key).await {
        Ok(_) => HttpResponse::Ok().body("Key deleted"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[delete("/clear")]
async fn redis_clear() -> impl Responder {
    match RedisService::clear_all().await {
        Ok(_) => HttpResponse::Ok().body("All keys deleted"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[derive(Deserialize)]
struct UpdatePayload {
    value: String,
    ttl: Option<usize>, // opsional kalau mau langsung set TTL juga
}

#[put("/update/{key}")]
async fn redis_update_key(
    key: web::Path<String>,
    payload: web::Json<UpdatePayload>,
) -> impl Responder {
    match RedisService::update_key(&key, &payload.value, payload.ttl).await {
        Ok(_) => HttpResponse::Ok().body("Key updated"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

struct RedisService;

impl RedisService {
    pub async fn list_keys(pattern: &str) -> Result<Vec<String>, redis::RedisError> {
        let mut conn = REDIS_CLIENT
            .get()
            .expect("Redis not initialized")
            .clone();

        let mut keys: Vec<String> = Vec::new();
        let mut iter = conn.scan_match::<String, String>(pattern.to_owned())?;
        while let Some(key) = iter.next() {
            keys.push(key);
        }

        Ok(keys)
    }

    pub async fn get_key(key: &str) -> Result<Option<String>, redis::RedisError> {
        let mut conn = REDIS_CLIENT
            .get()
            .expect("Redis not initialized")
            .clone();

        conn.get(key)
    }

    pub async fn delete_key(key: &str) -> Result<(), redis::RedisError> {
        let mut conn = REDIS_CLIENT
            .get()
            .expect("Redis not initialized")
            .clone();

        conn.del::<_, ()>(key)
    }

    pub async fn clear_all() -> Result<(), redis::RedisError> {
        let mut conn = REDIS_CLIENT
            .get()
            .expect("Redis not initialized")
            .clone();

        conn.flushdb::<()>()
    }

    pub async fn update_key(
        key: &str,
        value: &str,
        ttl: Option<usize>,
    ) -> Result<(), redis::RedisError> {
        let mut conn = REDIS_CLIENT
            .get()
            .expect("Redis not initialized")
            .clone();

        match ttl {
            Some(seconds) => conn.set_ex::<_, _, ()>(key, value, seconds.try_into().unwrap())?,
            None => conn.set::<_, _, ()>(key, value)?,
        }

        Ok(())
    }
}
