use std::collections::HashMap;

use actix_web::{get, web, HttpResponse, Responder, Scope};

use crate::services::mail_service::MailService;

pub fn mail_scope() -> Scope {
    
    web::scope("/email")
        .service(preview_template)
}

#[get("/preview")]
pub async fn preview_template(query: web::Query<HashMap<String, String>>) -> impl Responder {
    let template = query.get("template").map_or("activation", |v| v).to_string();

    // Dummy request
    let mut request = HashMap::new();
    request.insert("FirstName".to_string(), "Budi".to_string());
    request.insert("ActivationURL".to_string(), "https://example.com/activate/abc123".to_string());
    request.insert("CompanyName".to_string(), "PT. MICRO PIRANTI COMPUTER".to_string());
    request.insert("subject".to_string(), "Verifikasi Akun Anda".to_string());
    request.insert("email".to_string(), "budi@example.com".to_string());

    match MailService::preview(&template, &request) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Render error: {}", e)),
    }
}