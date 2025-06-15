use actix_web::{get, web, HttpRequest, HttpResponse, Scope};
use crate::{middleware::{jwt_session::{validate_jwt, Claims}, model::ActionResult}, services::{auth_service::AuthService, option_service::OptionService}};

pub fn option_scope() -> Scope {
    
    web::scope("/options")
        .service(get_question_npwp)
        .service(get_options_city)
        .service(get_options)
}

const APP_NAME: &str = "snakesystem-api";

#[get("/{code}")]
pub async fn get_options(req: HttpRequest, path: web::Path<String>, query: web::Query<std::collections::HashMap<String, String>>) -> HttpResponse {
    let code = path.into_inner();
    let keyword = query.get("keyword").map(|s| s.as_str());

    // 1. Definisikan kode yang boleh diakses tanpa login
    let public_codes = vec![
        "sex", 
        "sales"
    ];

    // 2. Jika code termasuk public_codes, langsung panggil OptionService tanpa validasi
    if public_codes.iter().any(|&pc| pc == code) {
        // Langsung ambil data dari OptionService
        let list_result: ActionResult<serde_json::Value, String> = OptionService::get_options(&code, keyword).await;

        return match list_result {
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
            res => HttpResponse::BadRequest().json(serde_json::json!({
                "error": res.message
            })),
        };
    }

    // 3. Kalau bukan public_codes, wajib login → cek cookie + JWT + session
    // Buat ActionResult<Claims, _> untuk menampung hasil sesi
    let mut auth_res: ActionResult<Claims, String> = ActionResult::default();
    // 3a. Ambil cookie "token" (nama cookie diasumsikan APP_NAME)
    let token_cookie = req.cookie(APP_NAME);

    let token = match token_cookie {
        Some(cookie) => cookie.value().to_string(),
        None => {
            auth_res.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": auth_res.error
            }));
        }
    };

    // 3b. Validasi JWT; misal validate_jwt(&token) → Result<Claims, Err>
    let claims = match validate_jwt(&token) {
        Ok(claims) => claims,
        Err(err) => {
            auth_res.error = Some(err.to_string());
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": auth_res.error
            }));
        }
    };

    // 3c. Cek session lewat AuthService
    // Asumsikan signature: check_session(claims: Claims, access: String, refresh: String, flag1: bool, flag2: bool, flag3: bool, cookie_name: &str)
    auth_res = AuthService::check_session(
        claims.clone(),
        token.clone(),  // bisa saja kamu pakai token yang sama untuk access & refresh 
        token.clone(),
        false,
        false,
        true,
        APP_NAME,
    )
    .await;

    // 3d. Tangani response dari check_session
    if let Some(err_msg) = &auth_res.error {
        // Error di server
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err_msg
        }));
    }
    if !auth_res.result {
        // Sesi tidak valid (misal token expired)
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": auth_res.message
        }));
    }

    // Jika valid, lanjut panggil OptionService
    let list_result: ActionResult<serde_json::Value, String> = OptionService::get_options(&code, keyword).await;

    match list_result {
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
        res => HttpResponse::BadRequest().json(serde_json::json!({
            "error": res.message
        })),
    }
}

#[get("/city")]
pub async fn get_options_city(req: HttpRequest, query: web::Query<std::collections::HashMap<String, String>>) -> HttpResponse {
    let keyword = query.get("keyword").map(|s| s.as_str());

    // 3. Kalau bukan public_codes, wajib login → cek cookie + JWT + session
    // Buat ActionResult<Claims, _> untuk menampung hasil sesi
    let mut auth_res: ActionResult<Claims, String> = ActionResult::default();
    // 3a. Ambil cookie "token" (nama cookie diasumsikan APP_NAME)
    let token_cookie = req.cookie(APP_NAME);

    let token = match token_cookie {
        Some(cookie) => cookie.value().to_string(),
        None => {
            auth_res.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": auth_res.error
            }));
        }
    };

    // 3b. Validasi JWT; misal validate_jwt(&token) → Result<Claims, Err>
    let claims = match validate_jwt(&token) {
        Ok(claims) => claims,
        Err(err) => {
            auth_res.error = Some(err.to_string());
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": auth_res.error
            }));
        }
    };

    // 3c. Cek session lewat AuthService
    // Asumsikan signature: check_session(claims: Claims, access: String, refresh: String, flag1: bool, flag2: bool, flag3: bool, cookie_name: &str)
    auth_res = AuthService::check_session(
        claims.clone(),
        token.clone(),  // bisa saja kamu pakai token yang sama untuk access & refresh 
        token.clone(),
        false,
        false,
        true,
        APP_NAME,
    )
    .await;

    // 3d. Tangani response dari check_session
    if let Some(err_msg) = &auth_res.error {
        // Error di server
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err_msg
        }));
    }
    if !auth_res.result {
        // Sesi tidak valid (misal token expired)
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": auth_res.message
        }));
    }

    // Jika valid, lanjut panggil OptionService
    let list_result: ActionResult<serde_json::Value, String> = OptionService::get_options_city(keyword).await;

    match list_result {
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
        res => HttpResponse::BadRequest().json(serde_json::json!({
            "error": res.message
        })),
    }
}

#[get("/npwp")]
pub async fn get_question_npwp(req: HttpRequest) -> HttpResponse {

    let mut auth_res: ActionResult<Claims, String> = ActionResult::default();
    // 3a. Ambil cookie "token" (nama cookie diasumsikan APP_NAME)
    let token_cookie = req.cookie(APP_NAME);

    let token = match token_cookie {
        Some(cookie) => cookie.value().to_string(),
        None => {
            auth_res.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": auth_res.error
            }));
        }
    };

    // 3b. Validasi JWT; misal validate_jwt(&token) → Result<Claims, Err>
    let claims = match validate_jwt(&token) {
        Ok(claims) => claims,
        Err(err) => {
            auth_res.error = Some(err.to_string());
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": auth_res.error
            }));
        }
    };

    auth_res = AuthService::check_session(
        claims.clone(),
        token.clone(),  // bisa saja kamu pakai token yang sama untuk access & refresh 
        token.clone(),
        false,
        false,
        true,
        APP_NAME,
    )
    .await;

    if let Some(err_msg) = &auth_res.error {
        // Error di server
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err_msg
        }));
    }
    if !auth_res.result {
        // Sesi tidak valid (misal token expired)
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": auth_res.message
        }));
    }

    let list_result: ActionResult<Vec<serde_json::Value>, String> = OptionService::get_question_npwp().await;

    match list_result {
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
        res => HttpResponse::BadRequest().json(serde_json::json!({
            "error": res.message
        })),
    }
}

