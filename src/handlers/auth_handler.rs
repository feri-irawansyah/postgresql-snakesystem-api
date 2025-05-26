use actix_web::{cookie::{time, Cookie, SameSite}, post, web, HttpRequest, HttpResponse, Responder, Scope};
use sqlx::PgPool;
use validator::Validate;

use crate::{middleware::{jwt_session::{create_jwt, Claims}, model::{ActionResult, LoginRequest, RegisterRequest}}, services::{auth_service::AuthService, generic_service::GenericService}};

const APP_NAME: &str = "snakesystem-web-api";

pub fn auth_scope() -> Scope {
    
    web::scope("/auth")
        .service(login)
        .service(register)
}

#[post("/login")]
async fn login(req: HttpRequest, connection: web::Data<PgPool>, request: web::Json<LoginRequest>) -> impl Responder {

    let mut result: ActionResult<Claims, _> = AuthService::login(connection.get_ref(), request.into_inner(), &req, APP_NAME).await;

    let token_cookie = req.cookie("snakesystem").map(|c| c.value().to_string()).unwrap_or_default();

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(response)
        }, // Jika error, HTTP 500
        response if response.result => {
            if let Some(user) = &response.data {
                // ✅ Buat token JWT
                match create_jwt(user.clone()) {
                    Ok(token) => {
                        // ✅ Simpan token dalam cookie
                        // result = AuthService::check_session(connection.get_ref(), user.clone(), token.clone(), token_cookie.clone(), false, false, false).await;

                        // ✅ Jika berhasil, kembalikan JSON response
                        // if !result.result {
                        //     return HttpResponse::InternalServerError().json(result);
                        // }
                            
                        let cookie = Cookie::build("snakesystem", token_cookie.is_empty().then(|| token.clone()).unwrap_or(token_cookie.clone()))
                            .path("/")
                            .http_only(true)
                            .same_site(SameSite::None) // ❗ WAJIB None agar cookie cross-site
                            .secure(true)     
                            .expires(time::OffsetDateTime::now_utc() + time::Duration::days(2)) // Set expired 2 hari
                            .finish();

                        return HttpResponse::Ok()
                            .cookie(cookie)
                            .json(response);
                    }
                    Err(err) => {
                        println!("❌ Failed to create JWT: {}", err);
                        return HttpResponse::InternalServerError().json(response);
                    }
                }
            }

            HttpResponse::BadRequest().json(response) // Jika tidak ada user, return 400
        },
        response => HttpResponse::BadRequest().json(response), // Jika gagal login, HTTP 400
    }
}

#[post("/register")]
async fn register(req: HttpRequest, connection: web::Data<PgPool>, mut request: web::Json<RegisterRequest>) -> impl Responder {

    if let Err(err) = request.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        }));
    }

    request.app_ipaddress = GenericService::get_ip_address(&req);

    let result: ActionResult<String, String> = AuthService::register(&connection, request.into_inner()).await;

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