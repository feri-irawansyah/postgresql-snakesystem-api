use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Scope};
use validator::Validate;

use crate::{middleware::model::{ActionResult, NewNoteRequest, NewPortfolioRequest, NewSkillRequest, Notes, Portfolio, Skill, UpdateNoteRequest, UpdatePortfolioRequest, UpdateSkillRequest}, services::{generic_service::GenericService, library_service::LibraryService}};

pub fn library_scope() -> Scope {
    
    web::scope("/library").configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_library)
        .service(create_libary)
        .service(update_libary)
        .service(create_skill)
        .service(update_skill)
        .service(get_skill)
        .service(create_portfolio)
        .service(update_portfolio)
        .service(get_portfolio);
}

#[post("/create")]
async fn create_libary(req: HttpRequest, request: web::Json<NewNoteRequest>) -> impl Responder {
    if let Err(err) = request.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "result": false,
            "message": "Invalid request",
            "error": err
        }));
    }

    // ambil ownership dan ubah
    let mut request = request.into_inner();
    if request.slug.is_none() || request.slug.as_ref().unwrap().trim().is_empty() {
        request.slug = Some(GenericService::slugify(&request.title));
    }

    let result: ActionResult<String, String> = LibraryService::create_library(req, request).await;

    if !result.result {
        return HttpResponse::InternalServerError().json(result);
    }

    HttpResponse::Ok().json(result)
}

#[post("/update")]
async fn update_libary(req: HttpRequest, request: web::Json<UpdateNoteRequest>) -> impl Responder {
    if let Err(err) = request.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "result": false,
            "message": "Invalid request",
            "error": err
        }));
    }

    // ambil ownership dan ubah
    let mut request = request.into_inner();
    if request.slug.is_none() || request.slug.as_ref().unwrap().trim().is_empty() {
        request.slug = Some(GenericService::slugify(&request.title));
    }

    let result: ActionResult<String, String> = LibraryService::update_library(req, request).await;

    if !result.result {
        return HttpResponse::InternalServerError().json(result);
    }

    HttpResponse::Ok().json(result)
}

#[get("/get-library/{slug}")]
async fn get_library(slug: web::Path<String>) -> impl Responder {

    if slug.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid slug url".to_string()
        }));
    }

    let result: ActionResult<Notes, String> = LibraryService::get_library(slug.to_string()).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": response.error
            }))
        }, // Jika error, HTTP 500
        response if response.result => HttpResponse::Ok().json(serde_json::json!({
            "data": response.data
        })), // Jika berhasil, HTTP 200
        response => HttpResponse::BadRequest().json(serde_json::json!({ 
            "error": response.message
         })), // Jika gagal, HTTP 400
    }
}

#[post("/create-skill")]
async fn create_skill(request: web::Json<NewSkillRequest>) -> impl Responder {
    if let Err(err) = request.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "result": false,
            "message": "Invalid request",
            "error": err
        }));
    }

    let result: ActionResult<String, String> = LibraryService::create_skill(request.into_inner()).await;

    if !result.result {
        return HttpResponse::InternalServerError().json(result);
    }

    HttpResponse::Ok().json(result)
}

#[post("/update-skill")]
async fn update_skill(request: web::Json<UpdateSkillRequest>) -> impl Responder {
    if let Err(err) = request.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "result": false,
            "message": "Invalid request",
            "error": err
        }));
    }

    // ambil ownership dan ubah
    let request = request.into_inner();

    let result: ActionResult<String, String> = LibraryService::update_skill(request).await;

    if !result.result {
        return HttpResponse::InternalServerError().json(result);
    }

    HttpResponse::Ok().json(result)
}

#[get("/get-skill/{skill_id}")]
async fn get_skill(skill_id: web::Path<i32>) -> impl Responder {

    if skill_id == 0.into() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid skill url".to_string()
        }));
    }

    let result: ActionResult<Skill, String> = LibraryService::get_skill(*skill_id).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": response.error
            }))
        }, // Jika error, HTTP 500
        response if response.result => HttpResponse::Ok().json(serde_json::json!({
            "data": response.data
        })), // Jika berhasil, HTTP 200
        response => HttpResponse::BadRequest().json(serde_json::json!({ 
            "error": response.message
         })), // Jika gagal, HTTP 400
    }
}

#[post("/create-portfolio")]
async fn create_portfolio(request: web::Json<NewPortfolioRequest>) -> impl Responder {
    if let Err(err) = request.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "result": false,
            "message": "Invalid request",
            "error": err
        }));
    }

    let result: ActionResult<String, String> = LibraryService::create_portfolio(request.into_inner()).await;

    if !result.result {
        return HttpResponse::InternalServerError().json(result);
    }

    HttpResponse::Ok().json(result)
}

#[post("/update-portfolio")]
async fn update_portfolio(request: web::Json<UpdatePortfolioRequest>) -> impl Responder {
    if let Err(err) = request.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "result": false,
            "message": "Invalid request",
            "error": err
        }));
    }

    // ambil ownership dan ubah
    let request = request.into_inner();

    let result: ActionResult<String, String> = LibraryService::update_portfolio(request).await;

    if !result.result {
        return HttpResponse::InternalServerError().json(result);
    }

    HttpResponse::Ok().json(result)
}

#[get("/get-portfolio/{portfolio_id}")]
async fn get_portfolio(portfolio_id: web::Path<i32>) -> impl Responder {

    if portfolio_id == 0.into() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid portfolio url".to_string()
        }));
    }

    let result: ActionResult<Portfolio, String> = LibraryService::get_portfolio(*portfolio_id).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": response.error
            }))
        }, // Jika error, HTTP 500
        response if response.result => HttpResponse::Ok().json(serde_json::json!({
            "data": response.data
        })), // Jika berhasil, HTTP 200
        response => HttpResponse::BadRequest().json(serde_json::json!({ 
            "error": response.message
         })), // Jika gagal, HTTP 400
    }
}
