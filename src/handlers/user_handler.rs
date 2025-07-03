use actix_web::{get, web, HttpRequest, HttpResponse, Responder, Scope};

use crate::{middleware::{jwt_session::validate_jwt, model::ActionResult}, services::{auth_service::AuthService, user_service::UserService}};

const APP_NAME: &str = "snakesystem-api";

pub fn user_scope() -> Scope {
    
    web::scope("/user").configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user);
}

#[get("/data")]
async fn get_user(req: HttpRequest) -> impl Responder {

    let mut result: ActionResult<_, _> = ActionResult::default();

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

                    let data = UserService::get_user(claims.clone()).await;
                    
                    match data {
                        res if res.error.is_some() => {
                            HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": res.error
                            }))
                        }
                        res if res.result => {
                            HttpResponse::Ok().json(serde_json::json!({
                                "data": res.data
                            }))
                        }
                        res => {
                            HttpResponse::BadRequest().json(serde_json::json!({
                                "error": res.message
                            }))
                        }
                    }
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