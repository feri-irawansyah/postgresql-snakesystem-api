use actix_web::{cookie::{time, Cookie, SameSite}, post, get, web, HttpRequest, HttpResponse, Responder, Scope};
use oauth2::{AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, DeviceAuthorizationUrl, PkceCodeChallenge, RedirectUrl, TokenResponse, TokenUrl, basic::BasicClient};
use validator::Validate;

use crate::{AppState, SECRETS, middleware::{
    jwt_session::{Claims, create_jwt, validate_jwt}, 
    model::{ActionResult, ChangePasswordRequest, LoginRequest, RegisterRequest, ResetPasswordRequest}}, services::{auth_service::AuthService, generic_service::GenericService
}};

const APP_NAME: &str = "snakesystem-api";

pub fn auth_scope() -> Scope {
    
    web::scope("/auth").configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(login)
        .service(check_session)
        .service(register)
        .service(activation)
        .service(reset_password)
        .service(change_password)
        .service(logout)
        .service(google_login)
        .service(google_callback);
}

#[post("/login")]
async fn login(req: HttpRequest, request: web::Json<LoginRequest>) -> impl Responder {
    
    let mut result: ActionResult<Claims, _> = AuthService::login(request.into_inner(), &req, APP_NAME).await;

    // let token_cookie = req.cookie(APP_NAME).map(|c| c.value().to_string()).unwrap_or_default();

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
                        if result.error.is_some() {
                            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": result.error }));
                        }

                        if result.result {
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
                            
                        if !result.result {
                            return HttpResponse::BadRequest().json(serde_json::json!({ "error": result.message }));
                        }
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

#[post("/reset-password")]
async fn reset_password(request: web::Json<ResetPasswordRequest>) -> impl Responder {

    if let Err(err) = request.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        }));
    }

    let result: ActionResult<String, String> = AuthService::reset_password(request.into_inner()).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": response.error
            }))
        }, // Jika error, HTTP 500
        response if response.result => HttpResponse::Ok().json(serde_json::json!({
            "data": response.message
        })), // Jika berhasil, HTTP 200
        response => HttpResponse::BadRequest().json(serde_json::json!({ "error": response.message })), // Jika gagal, HTTP 400
    }
}

#[post("/change-password")]
async fn change_password(request: web::Json<ChangePasswordRequest>) -> impl Responder {

    if let Err(err) = request.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        }));
    }

    let result: ActionResult<String, String> = AuthService::change_password(request.into_inner()).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": response.error
            }))
        }, // Jika error, HTTP 500
        response if response.result => HttpResponse::Ok().json(serde_json::json!({
            "data": response.message
        })), // Jika berhasil, HTTP 200
        response => HttpResponse::BadRequest().json(serde_json::json!({ "error": response.message })), // Jika gagal, HTTP 400
    }
}

#[post("/logout")]
async fn logout(req: HttpRequest) -> impl Responder {

    let mut result: ActionResult<Claims, _> = ActionResult::default();

    // Ambil cookie "token"
    let token_cookie = req.cookie(APP_NAME);

    println!("token_cookie: {:#?}", token_cookie);

    // Cek apakah token ada di cookie
    let token = match token_cookie {
        Some(cookie) => cookie.value().to_string(),
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        }
    };

    match validate_jwt(&token) {
        Ok(claims) => {
            result = AuthService::check_session(claims.clone(), token.clone(), token.clone(), true, false, false, APP_NAME).await;

            match result {
                response if response.error.is_some() => {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": response.error
                    }))
                },
                response if response.result => {
                    
                    let cookie = Cookie::build(APP_NAME, "")
                    .path("/")
                    .http_only(true)
                    .same_site(SameSite::None) // ❗ WAJIB None agar cookie cross-site
                    .secure(true)     
                    .expires(time::OffsetDateTime::now_utc() + time::Duration::days(2)) // Set expired 2 hari
                    .finish();

                    return HttpResponse::Ok()
                        .cookie(cookie)
                        .json(serde_json::json!({ "data": response.message }));
                    
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

#[get("/google/login")]
async fn google_login(data: web::Data<AppState>) -> impl Responder {

    let secrets = SECRETS.get().expect("SECRETS not initialized");
    let client_id = secrets.get("GOOGLE_ID").expect("secret was not found");
    let client_secret = secrets.get("GOOGLE_SECRET").expect("secret was not found");
    let domain = secrets.get("DOMAIN").expect("secret was not found");

    let redirect_url = format!("{}/api/v1/auth/google/callback", domain);

    let client = BasicClient::new(ClientId::new(client_id.to_string()),)
    .set_client_secret(ClientSecret::new(client_secret.to_string()))
    .set_auth_uri(AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string()).unwrap())
    .set_token_uri(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap())
    // Set the URL the user will be redirected to after the authorization process.
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap());

    // PKCE
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // URL + CSRF Token
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        // .add_scope(oauth2::Scope::new("read".to_string()))
        .add_scope(oauth2::Scope::new("openid".to_string()))
        .add_scope(oauth2::Scope::new("email".to_string()))
        .add_scope(oauth2::Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Simpan csrf + pkce ke AppState (Arc<Mutex<...>>)
    {
        let mut store = data.oauth_store.lock().unwrap();
        store.insert(csrf_token.secret().to_string(), (csrf_token.clone(), pkce_verifier));
    }

    HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}

#[get("/google/callback")]
async fn google_callback(query: web::Query<std::collections::HashMap<String,String> >, data: web::Data<AppState>) -> impl Responder {
    let secrets = SECRETS.get().expect("SECRETS not initialized");
    let client_id = secrets.get("GOOGLE_ID").expect("secret was not found");
    let client_secret = secrets.get("GOOGLE_SECRET").expect("secret was not found");
    let domain = secrets.get("DOMAIN").expect("secret was not found");

    let redirect_url = format!("{}/api/v1/auth/google/callback", domain);

    let client = BasicClient::new(ClientId::new(client_id.to_string()))
        .set_client_secret(ClientSecret::new(client_secret.to_string()))
        .set_device_authorization_url(DeviceAuthorizationUrl::new("https://oauth2.googleapis.com/device/code".into()).unwrap())
        .set_auth_uri(AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into()).unwrap())
        .set_token_uri(TokenUrl::new("https://oauth2.googleapis.com/token".into()).unwrap())
        .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap());

    // ambil query params (cek ada atau nggak)
    let code = match query.get("code") {
        Some(c) => c.clone(),
        None => return HttpResponse::BadRequest().body("Missing code"),
    };
    let state = match query.get("state") {
        Some(s) => s.clone(),
        None => return HttpResponse::BadRequest().body("Missing state"),
    };

    // ambil & hapus pair dari store (one-time use)
    let pair_opt = {
        let mut store = data.oauth_store.lock().unwrap();
        store.remove(&state)
    };

    let (csrf_token, pkce_verifier) = match pair_opt {
        Some(pair) => pair,
        None => return HttpResponse::BadRequest().body("Invalid or expired state"),
    };

    // verifikasi CSRF
    if csrf_token.secret() != &state {
        return HttpResponse::Unauthorized().body("Invalid CSRF token");
    }

    // exchange code -> token (async)
    let http_client = oauth2::reqwest::ClientBuilder::new()
        .redirect(oauth2::reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let token_result = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        .await;

    match token_result {
        Ok(token) => {
            let access_token = token.access_token().secret();
            HttpResponse::Ok().body(format!("Access token: {}", access_token))
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Error: {:?}", err)),
    }
}