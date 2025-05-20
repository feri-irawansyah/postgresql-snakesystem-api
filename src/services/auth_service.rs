use actix_web::{web, HttpRequest};
use sqlx::PgPool;

use crate::contexts::{crypto::encrypt_text, jwt_session::Claims, model::{ActionResult, LoginRequest}};

pub struct AuthService;

impl AuthService {
    pub async fn login(
        connection: web::Data<PgPool>,
        request: LoginRequest,
        req: HttpRequest,
        app_name: &str,
    ) -> ActionResult<Claims, String> {
        let mut result: ActionResult<Claims, String> = ActionResult::default();
        let enc_password = encrypt_text(request.password.unwrap_or_default());

        let query_result = sqlx::query(
            r#"
            SELECT 
                "AuthUserNID", 
                "Email", 
                "Handphone", 
                "disableLogin", 
                "Picture", 
                "RegisterDate"
            FROM "AuthUser"
            WHERE "Email" = $1 AND "Password" = $2
            "#
        )
        .bind(request.email.unwrap_or_default())
        .bind(enc_password)
        .fetch_optional(connection.get_ref())
        .await;

        match query_result {
            Ok(Some(row)) => {
                result.result = true;
                result.message = "Login success".to_string();
            }
            Ok(None) => {
                result.message = "Login failed".to_string();
            }
            Err(err) => {
                result.error = format!("Query failed: {:?}", err).into();
            }
        }

        result
    }
}
