use std::collections::HashMap;

use handlebars::Handlebars;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};

use crate::{middleware::model::ActionResult, SECRETS};

pub struct MailService;

impl MailService {
    pub async fn send(request: HashMap<String, Option<String>>, template: &str) -> ActionResult<String, String> {
        let mut result: ActionResult<String, String> = ActionResult::default();
        
        let secrets = SECRETS.get().expect("SECRETS not initialized");
        
        let smtp_username = secrets.get("SMTP_USER").expect("secret was not found");
        let smtp_password = secrets.get("SMTP_PASSWORD").expect("secret was not found");
        let smtp_server = secrets.get("SMTP_SERVER").expect("secret was not found");
        let email_from = secrets.get("EMAIL_FROM").expect("secret was not found");

        // Baca template
        let template_str = match template {
            "activation" => include_str!("../../templates/activation.hbs"),
            "reset-password" => include_str!("../../templates/reset_password.hbs"),
            _ => panic!("Template not found"),
        };

        // Setup handlebars
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string(template, template_str).unwrap();

        let html_body = handlebars.render(template, &request).unwrap();

        let to_email = request.get("email")
            .and_then(|v| v.as_ref())
            .ok_or("Missing email").unwrap()
            .parse()
            .map_err(|_| "Invalid email address").unwrap();

        let subject = request.get("subject")
            .and_then(|v| v.as_ref())
            .ok_or("Missing subject").unwrap();


        let email = Message::builder()
            .from(email_from.parse().unwrap())
            .to(to_email)
            .subject(subject)
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(html_body)
            .unwrap();

        let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());

        let mailer = SmtpTransport::relay(&smtp_server)
            .unwrap()
            .credentials(creds)
            .build();

        match mailer.send(&email) {
            Ok(res) => {
                println!("Email sent: {:#?}", res);
                result.result = true;
                result.message = "Email sent successfully!".to_string();
            }
            Err(e) => {
                eprintln!("Failed to send email: {e}");
                result.result = false;
                result.error = Some(e.to_string());
            }
        }

        return result;
    }

     pub fn preview(template: &str, data: &HashMap<String, String>) -> Result<String, String> {
        // Load template
        let template_str = match template {
            "activation" => include_str!("../../templates/activation.hbs"),
            "reset-password" => include_str!("../../templates/reset_password.hbs"),
            _ => return Err("Template not found".to_string()),
        };

        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string(template, template_str)
            .map_err(|e| e.to_string())?;

        handlebars
            .render(template, data)
            .map_err(|e| e.to_string())
    }
}