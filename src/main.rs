use actix_cors::Cors;
use actix_web::{get, http, web::{self, route, Data, ServiceConfig}, Responder};
use docs::swagger::{health_check, Swagger};
use handlers::auth_handler::auth_scope;
use services::generic_service::GenericService;
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::SecretStore;
use sqlx::{PgPool, postgres::PgPoolOptions};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[get("/")]
async fn hello_world(db: Data<PgPool>) -> impl Responder {
    // Cek koneksi DB
    let row = sqlx::query("SELECT * FROM email_history").fetch_all(&**db);
        
    format!("Hello World! DB returns: {:?}", row.await.inspect(|v| println!("{:?}", v)))
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
}
mod handlers {
    pub mod auth_handler;
}
mod utils {
    pub mod api_docs;
    pub mod validation;
}

mod docs {
    pub mod swagger;
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let db_url = secrets.get("DATABASE_URL").expect("DB URL not found");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    let config = move |cfg: &mut ServiceConfig| {
        let cors = Cors::default()
            .allow_any_origin() // Atau pakai .allow_any_origin() dynamic app https only
            // .allowed_origin("http://localhost:5173") // url development
            // .allowed_origin("https://snakesystem.github.io") // url production
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec![http::header::CONTENT_TYPE])
            .max_age(3600)
            .supports_credentials();
        
        cfg
        .service(health_check)
        .service(
            web::scope("/api/v1")
            .wrap(cors)
            .service(auth_scope())
        )
        .service(
            SwaggerUi::new("/docs/{_:.*}")
                .url("/api-docs/openapi.json", Swagger::openapi())
        )
        .app_data(web::Data::new(pool.clone()))
        .app_data(web::Data::new(secrets.clone()))
        .app_data(web::JsonConfig::default().error_handler(GenericService::json_error_handler))
        .default_service(route().to(GenericService::not_found));
    };

    Ok(config.into())
}