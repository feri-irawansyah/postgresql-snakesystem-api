use actix_web::{get, web::{Data, ServiceConfig}, Responder};
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::SecretStore;
use sqlx::{PgPool, postgres::PgPoolOptions};

#[get("/")]
async fn hello_world(db: Data<PgPool>) -> impl Responder {
    // Cek koneksi DB
    let row = sqlx::query("SELECT * FROM email_history").fetch_all(&**db);
        
    format!("Hello World! DB returns: {:?}", row.await.inspect(|v| println!("{:?}", v)))
}


mod contexts {
    pub mod crypto;
    pub mod jwt_session;
    pub mod socket;
    pub mod model;
}

mod services {
    pub mod auth_service;
    pub mod mail_service;
}

mod handlers {

}

mod utils {
    pub mod api_docs;
    pub mod validation;
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let db_url = secrets.get("DATABASE_URL").expect("DB URL not found");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(Data::new(pool.clone()));
        cfg.service(hello_world);
    };

    Ok(config.into())
}