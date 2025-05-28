use actix_web::{cookie::{time, Cookie, SameSite}, post, get, web, HttpRequest, HttpResponse, Responder, Scope};
use validator::Validate;

use crate::{middleware::{
    jwt_session::{create_jwt, validate_jwt, Claims}, 
    model::{ActionResult, LoginRequest, RegisterRequest}},
    services::{auth_service::AuthService, generic_service::GenericService
}};

const APP_NAME: &str = "snakesystem-api";

pub fn auth_scope() -> Scope {
    
    web::scope("/auth")
        .service(login)
        .service(check_session)
        .service(register)
        .service(activation)
}

#[post("/login")]
async fn login(req: HttpRequest, request: web::Json<LoginRequest>) -> impl Responder {
    
    let mut result: ActionResult<Claims, _> = AuthService::login(request.into_inner(), &req).await;

    let token_cookie = req.cookie(APP_NAME).map(|c| c.value().to_string()).unwrap_or_default();

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": response.error
            }))
        }, // Jika error, HTTP 500
        response if response.result => {
            if let Some(user) = &response.data {
                // ✅ Buat token JWT
                match create_jwt(user.clone()) {
                    Ok(token) => {
                        // ✅ Simpan token dalam cookie
                        result = AuthService::check_session(user.clone(), token.clone(), "".to_string(), false, false, false, APP_NAME).await;

                        // ✅ Jika berhasil, kembalikan JSON response
                        if !result.result {
                            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": result.error }));
                        }
                            
                        let cookie = Cookie::build(APP_NAME, token)
                            .path("/")
                            .http_only(true)
                            .same_site(SameSite::None) // ❗ WAJIB None agar cookie cross-site
                            .secure(true)     
                            .expires(time::OffsetDateTime::now_utc() + time::Duration::days(2)) // Set expired 2 hari
                            .finish();

                        return HttpResponse::Ok()
                            .cookie(cookie)
                            .json(serde_json::json!({ "data": result.message }));
                    }
                    Err(err) => {
                        println!("❌ Failed to create JWT: {}", err);
                        return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to create JWT" }));
                    }
                }
            }

            HttpResponse::BadRequest().json(response) // Jika tidak ada user, return 400
        },
        response => HttpResponse::BadRequest().json(serde_json::json!({ "error": response.message })), // Jika gagal login, HTTP 400
    }
}

#[get("/session")]
async fn check_session(req: HttpRequest) -> impl Responder {

    let mut result: ActionResult<Claims, _> = ActionResult::default();

    // Ambil cookie "token"
    let token_cookie = req.cookie(APP_NAME);

    // Cek apakah token ada di cookie
    let token = match token_cookie {
        Some(cookie) => cookie.value().to_string(),
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        }
    };

    // Validate token
    match validate_jwt(&token) {
        Ok(claims) => {
            result = AuthService::check_session(claims.clone(), token.clone(), token.clone(), false, false, true, APP_NAME).await;

            match result {
                response if response.error.is_some() => {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": response.error
                    }))
                },
                response if response.result => {
                    
                    HttpResponse::Ok().json({
                        serde_json::json!({
                            "data": Some(claims.clone())
                        })
                    })
                },
                response => HttpResponse::BadRequest().json(serde_json::json!({ "error": response.message })), // Jika gagal login, HTTP 400
            }
        },
        Err(err) => {
            result.error = Some(err.to_string());
            HttpResponse::Unauthorized().json(serde_json::json!({ "error": result.error }))
        },
    }
}

#[post("/register")]
async fn register(req: HttpRequest, mut request: web::Json<RegisterRequest>) -> impl Responder {

    if let Err(err) = request.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        }));
    }

    request.app_ipaddress = GenericService::get_ip_address(&req);

    let result: ActionResult<String, String> = AuthService::register(request.into_inner()).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": response.error
            }))
        }, // Jika error, HTTP 500
        response if response.result => HttpResponse::Ok().json(serde_json::json!({
            "data": response.message
        })), // Jika berhasil, HTTP 200
        response => HttpResponse::BadRequest().json(serde_json::json!({ 
            "error": response.message
         })), // Jika gagal, HTTP 400
    }
}

#[post("/activation/{activation_url}")]
async fn activation(activation_url: web::Path<String>) -> impl Responder {

    if activation_url.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid activation url".to_string()
        }));
    }

    let result: ActionResult<String, String> = AuthService::activation(activation_url.to_string()).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": response.error
            }))
        }, // Jika error, HTTP 500
        response if response.result => HttpResponse::Ok().json(serde_json::json!({
            "data": response.message
        })), // Jika berhasil, HTTP 200
        response => HttpResponse::BadRequest().json(serde_json::json!({ 
            "error": response.message
         })), // Jika gagal, HTTP 400
    }
}