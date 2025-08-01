use actix_cors::Cors;
use actix_web::{get, http, web::{self, route, ServiceConfig}, Responder};
use docs::swagger::{health_check, Swagger};
use handlers::{auth_handler::auth_scope, mail_handler::mail_scope, option_handler::option_scope};
use services::generic_service::GenericService;
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::SecretStore;
use sqlx::{PgPool, postgres::PgPoolOptions};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use once_cell::sync::OnceCell;

use crate::handlers::{data_handler::data_scope, library_handler::library_scope, user_handler::user_scope};

pub static CONNECTION: OnceCell<PgPool> = OnceCell::new();
pub static SECRETS: OnceCell<SecretStore> = OnceCell::new();

#[get("/")]
async fn hello_world() -> impl Responder {
        
    format!("Hello World! DB returns")
}

mod middleware {
    pub mod crypto;
    pub mod jwt_session;
    pub mod socket;
    pub mod model;
}
mod services {
    pub mod auth_service;
    pub mod mail_service;
    pub mod generic_service;
    pub mod user_service;
    pub mod option_service;
    pub mod library_service;
    pub mod data_service;
}
mod handlers {
    pub mod auth_handler;
    pub mod mail_handler;
    pub mod option_handler;
    pub mod user_handler;
    pub mod library_handler;
    pub mod data_handler;
}
mod utils {
    pub mod api_docs;
    pub mod validation;
}

mod docs {
    pub mod swagger;
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let db_url = secrets.get("DATABASE_URL").expect("DB URL not found");

    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .idle_timeout(std::time::Duration::from_secs(30))
        .max_lifetime(std::time::Duration::from_secs(60))
        .acquire_slow_threshold(std::time::Duration::from_secs(5))
        .connect(&db_url)
        .await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to connect to DB: {}", e);
                panic!("DB connection error");
            }
        };

    CONNECTION.set(pool.clone()).expect("Failed to set DB_POOL");
    SECRETS.set(secrets.clone()).unwrap_or_else(|_| panic!("Failed to set SECRETS"));

    let config = move |cfg: &mut ServiceConfig| {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec![http::header::CONTENT_TYPE])
            .max_age(3600)
            .supports_credentials();

        cfg.service(health_check)
            .service(
                web::scope("/api/v1")
                    .wrap(cors)
                    .service(mail_scope())
                    .service(option_scope())
                    .service(auth_scope())
                    .service(library_scope())
                    .service(data_scope())
                    .service(user_scope()),                   
            )
            .service(
                SwaggerUi::new("/docs/{_:.*}")
                    .url("/api-docs/openapi.json", Swagger::openapi()),
            )
            .app_data(web::Data::new(secrets.clone()))
            .app_data(web::JsonConfig::default().error_handler(GenericService::json_error_handler))
            .default_service(route().to(GenericService::not_found));
    };

    Ok(config.into())
}