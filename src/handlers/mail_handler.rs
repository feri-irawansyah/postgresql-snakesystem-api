use std::collections::HashMap;

use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use validator::Validate;

use crate::{middleware::model::{ActionResult, SendEmailRequest}, services::mail_service::MailService};

pub fn mail_scope() -> Scope {
    
    web::scope("/email")
        .service(preview_template)
        .service(send_email)
}

#[get("/preview")]
pub async fn preview_template(query: web::Query<HashMap<String, String>>) -> impl Responder {
    let template = query.get("template").map_or("activation", |v| v).to_string();

    // Dummy request
    let mut request = HashMap::new();
    request.insert("username".to_string(), "Budi".to_string());
    request.insert("activation_url".to_string(), "https://example.com/activate/abc123".to_string());
    request.insert("company_name".to_string(), "PT. SNAKESYSTEM".to_string());
    request.insert("subject".to_string(), "Verifikasi Akun Anda".to_string());
    request.insert("email".to_string(), "budi@example.com".to_string());
    request.insert("title".to_string(), "LAUNDERY".to_string());
    request.insert("otp_code".to_string(), "12345".to_string());

    match MailService::preview(&template, &request) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Render error: {}", e)),
    }
}

#[post("/send/{email_type}")]
pub async fn send_email(email_type: web::Path<String>, request: web::Json<SendEmailRequest>) -> impl Responder {

    if let Err(err) = request.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        }));
    }

    let mut mail_data = HashMap::new();
    mail_data.insert("email".to_string(), Some(request.email.clone()));
    mail_data.insert("front_url".to_string(), format!("{}/{}/{}", request.front_url, email_type, request.url_token).into());
    mail_data.insert("company_name".to_string(), Some(request.company_name.clone()));
    mail_data.insert("subject".to_string(), Some(request.subject.clone()));
    mail_data.insert("username".to_string(), Some(request.username.clone()));
    mail_data.insert("title".to_string(), Some(request.title.clone()));
    mail_data.insert("otp_code".to_string(), Some(request.otp_code.to_string()));

    let mail_result : ActionResult<String, String> = MailService::send(mail_data, &email_type).await;

    if mail_result.result {
        HttpResponse::Ok().json(serde_json::json!({
            "data": mail_result.message
        }))
    } else {
        HttpResponse::InternalServerError().json(serde_json::json!({ "error": mail_result.error }))
    }
}