use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Scope};
use validator::Validate;

use crate::{middleware::model::{ActionResult, NewNoteRequest, Notes, UpdateNoteRequest}, services::{generic_service::GenericService, library_service::LibraryService}};

pub fn library_scope() -> Scope {
    
    web::scope("/library")
        .service(get_library)
        .service(create_libary)
        .service(update_libary)
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