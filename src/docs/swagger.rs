use actix_web::{get, HttpResponse, Responder};
use utoipa::{OpenApi, ToSchema};

use crate::middleware::{jwt_session::Claims, model::LoginRequest};

#[utoipa::path(post, path = "/api/v1/auth/login", request_body = LoginRequest,
    responses(
        (status = 200, description = "Check Session", example = json!({
            "data": {
                "user_id": "1",
                "username": "admin",
                "email": "LXh4N@example.com",
                "company_id": "SS",
                "company_name": "Snake System Tech"
            }
        })),
        (status = 401, description = "Unauthorized", example = json!({
            "error": "Unauthorized"
        })),
        (status = 500, description = "Internal Server Error", example = json!({
            "error": "Internal Server Error"
        })),
        (status = 400, description = "Bad Request", example = json!({
            "error": "Bad Request"
        }))
    ),
    tag = "1. Authentiacation"
)]
#[allow(dead_code)]
pub fn login_docs() {}

// Health Check Docs

#[derive(serde::Serialize, ToSchema)]
struct HealthCheckResponse {
    data: String,
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Health Check Success", body = HealthCheckResponse, example = json!(HealthCheckResponse { data: "Welcome to the snakesystem app!".to_string(), }))
    ),
    tag = "0. Application Default Endpoints"
)]
#[get("/")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthCheckResponse {
        data: "Welcome to the snakesystem app!".to_string(),
    })
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Snakesystem API",
        description = "Dokumentasi untuk RESTful API SnakeSystem.\n\nSilakan gunakan token JWT untuk mengakses endpoint yang dilindungi.",
        version = "1.0.0"
    ),
    paths(
        login_docs,
        health_check
    ),
    components(
        schemas(
            LoginRequest,
            Claims),
    ),
    tags(
        (name = "0. Application Default Endpoints", description = "Default path application endpoints"),
        (name = "1. Authentiacation", description = "Authentication related endpoints"),
        (name = "2. Email Endpoints", description = "Mailer to send email related endpoints"),
        (name = "3. Library Endpoints", description = "Library endpoints to manage library data for Snakesystem Library"),
        (name = "4. Data Endpoints", description = "Data endpoints to manage generic data"),
        (name = "5. Generic Endpoints", description = "Generic endpoints to manage reusable url"),
    )
)]

pub struct Swagger;