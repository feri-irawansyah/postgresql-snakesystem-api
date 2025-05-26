use actix_web::HttpRequest;
use chrono::TimeZone;
use sqlx::PgPool;
use sqlx::Row;

use crate::middleware::model::RegisterRequest;
use crate::{middleware::{crypto::encrypt_text, jwt_session::Claims, model::{ActionResult, LoginRequest}}, services::generic_service::GenericService};

pub struct AuthService;

impl AuthService {
    pub async fn login(connection: &PgPool, request: LoginRequest, req: &HttpRequest, app_name: &str) -> ActionResult<Claims, String> {
        let mut result = ActionResult::default();
        let enc_password = encrypt_text(request.password.unwrap_or_default());

        let query_result = sqlx::query(
            r#"
            SELECT 
                user_id, 
                email, 
                hand_phone, 
                disabled, 
                picture, 
                register_date
            FROM auth_user
            WHERE email = $1 AND password = $2
            "#
        )
        .bind(request.email.clone().unwrap_or_default())
        .bind(enc_password)
        .fetch_optional(connection)
        .await;

        match query_result {
            Ok(Some(row)) => {
                result.result = true;
                result.message = format!("Welcome {}", row.try_get::<String, _>("email").unwrap_or_default());
                result.data = Some(Claims {
                    usernid: row.try_get::<i32, _>("user_id").unwrap_or(0),
                    email: row.try_get::<String, _>("email").unwrap_or_default(),
                    mobile_phone: row.try_get::<String, _>("hand_phone").unwrap_or_default(),
                    disabled_login: row.try_get::<bool, _>("disabled").unwrap_or(false),
                    picture: row.try_get::<Option<String>, _>("picture").unwrap_or_default(),
                    register_date: row
                        .try_get::<chrono::NaiveDateTime, _>("register_date")
                        .map(|d| d.and_utc())
                        .unwrap_or_else(|_| chrono::Utc.timestamp_opt(0, 0).unwrap()),

                    result: true,
                    expired_token: 0,
                    expired_date: "".to_string(),
                    exp: 0,
                    comp_name: Some(GenericService::get_device_name(req)),
                    ip_address: Some(GenericService::get_ip_address(req)),
                    app_name: Some(app_name.to_string()),
                });
            }
            Ok(None) => {
                result.message = "Invalid email or password".to_string();
            }
            Err(e) => {
                result.error = Some(format!("Database error: {}", e));
            }
        }

        result
    }

    pub async fn register(connection: &PgPool, request: RegisterRequest) -> ActionResult<String, String> {
        let mut result = ActionResult::default();

        let enc_password = encrypt_text(request.password.unwrap_or_default());

        let mut trans = match connection.begin().await {
            Ok(t) => t,
            Err(e) => {
                result.error = Some(format!("Database error: {}", e));
                return result;
            }            
        };

        if let Ok(Some(_)) = sqlx::query("SELECT email FROM users WHERE email = $1")
            .bind(&request.email)
            .fetch_optional(&mut *trans)
            .await {
                result.result = false;
                result.message = "Email already exists".into();
                return result;
        }

        let auto_nid: i32 = match sqlx::query(r#"
            INSERT INTO user_kyc 
            (email, mobile_phone, fullname, sales, stage, cif_nid, change_nid, pending_cif_nid,
            is_rejected, is_finished, is_revised, is_imported, save_time, last_update, save_ip_address)
            VALUES
            ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
            RETURNING autonid"#)
            .bind(&request.email)
            .bind(&request.mobile_phone)
            .bind(&request.full_name)
            .bind(&request.sales)
            .bind(1i32) // stage
            .bind(0i32) // cifnid
            .bind(0i32) // changenid
            .bind(0i32) // pendingcifnid
            .bind(false) // isrejected
            .bind(false) // isfinished
            .bind(false) // isrevised
            .bind(false) // isimported
            .bind(chrono::Utc::now().with_timezone(&chrono_tz::Asia::Jakarta))
            .bind(chrono::Utc::now().with_timezone(&chrono_tz::Asia::Jakarta))
            .bind(&request.app_ipaddress)
            .fetch_one(&mut *trans).await {
                Ok(row) => row.get("autonid"),
                Err(e) => {
                    result.error = Some(format!("Failed to insert UserKyc: {}", e));
                    return result;
                }
            };
        if let Err(e) = sqlx::query(r#"
            INSERT INTO users 
            (web_cif_id, email, handphone, activate_code, password, register_date,
            disable_login, otp_generated_link, otp_generated_link_date, picture, google_id, client_category)
            VALUES
            ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)"#)
            .bind(auto_nid)
            .bind(&request.email)
            .bind(&request.mobile_phone)
            .bind(GenericService::random_string(20))
            .bind(&enc_password)
            .bind(chrono::Utc::now().with_timezone(&chrono_tz::Asia::Jakarta))
            .bind(true)
            .bind(GenericService::random_string(70))
            .bind(chrono::Utc::now().with_timezone(&chrono_tz::Asia::Jakarta))
            .bind("") // picture
            .bind("") // sub
            .bind(&request.client_category)
            .execute(&mut *trans)
            .await {
                result.error = Some(format!("Failed to insert AuthUser: {}", e));
                return result;
            }

        println!("Jakarta: {}", chrono::Utc::now().with_timezone(&chrono_tz::Asia::Jakarta));
        // INSERT ke TableRequest
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO user_request (web_cif_nid, referal)
            VALUES ($1, $2)
            "#
        )
        .bind(auto_nid)
        .bind(&request.referal)
        .execute(&mut *trans)
        .await
        {
            result.error = Some(format!("Failed to insert TableRequest: {}", e));
            return result;
        }

        // Commit transaksi
        if let Err(e) = trans.commit().await {
            result.error = Some(format!("Failed to commit transaction: {}", e));
            return result;
        }

        result.result = true;
        result.message = "User registered successfully".into();

        return result;
    }

}