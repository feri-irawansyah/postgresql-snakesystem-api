use std::collections::HashMap;
use actix_web::HttpRequest;
use sqlx::PgPool;
use sqlx::Row;

use crate::middleware::jwt_session::validate_jwt;
use crate::middleware::model::RegisterRequest;
use crate::CONNECTION;
use crate::SECRETS;
use crate::{middleware::{crypto::encrypt_text, jwt_session::Claims, model::{ActionResult, LoginRequest}}, services::generic_service::GenericService};

use super::mail_service::MailService;

pub struct AuthService;

impl AuthService {
    pub async fn login(request: LoginRequest, req: &HttpRequest) -> ActionResult<Claims, String> {

        let connection: &PgPool = CONNECTION.get().unwrap();
        let mut result = ActionResult::default();
        let enc_password = encrypt_text(request.password.unwrap_or_default());

        let query_result = sqlx::query(
            r#"
            SELECT 
                B.autonid AS user_id, 
                B.fullname,
                A.email, 
                A.disable_login, 
                A.last_login, 
                A.picture, 
                A.register_date
            FROM users A
            LEFT JOIN user_kyc B ON A.web_cif_id = B.autonid
            WHERE A.email = $1 AND A.password = $2
            "#
        ).bind(request.email.clone().unwrap_or_default())
        .bind(enc_password)
        .fetch_one(connection)
        .await;

        match query_result {
            Ok(row) => {
                if row.get("disable_login") {
                    result.error = Some("Login disabled".to_string());
                    return result;
                }

                result.result = true;
                result.data = Some(Claims {
                    usernid: row.try_get::<i32, _>("user_id").unwrap_or(0),
                    fullname: row.try_get::<String, _>("fullname").unwrap_or_default(),
                    email: row.try_get::<String, _>("email").unwrap_or_default(),
                    disabled_login: row.try_get::<bool, _>("disable_login").unwrap_or(false),
                    picture: row.try_get::<Option<String>, _>("picture").unwrap_or_default(),
                    register_date: row.try_get("register_date").unwrap_or_else(|_| GenericService::get_timestamp()),
                    result: true,
                    expired_token: 0,
                    expired_date: "".to_string(),
                    exp: 0,
                    comp_name: Some(GenericService::get_device_name(req)),
                    ip_address: Some(GenericService::get_ip_address(req)),
                    app_name: Some("".to_string()),
                })
            }
            Err(e) => {
                result.message = format!("Incorrect email or password");
                println!("❌ Login Error: {}", e);
            }
        }

        result
    }

    pub async fn register(request: RegisterRequest) -> ActionResult<String, String> {

        let connection = CONNECTION.get().expect("DB_POOL not initialized");
        let secrets = SECRETS.get().expect("SECRETS not initialized");
        let front_url = secrets.get("FRONT_URL").expect("secret was not found");
        let enc_password = encrypt_text(request.password.unwrap_or_default());

        let mut result = ActionResult::default();

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
            RETURNING autonid;"#)
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
            .bind(GenericService::get_timestamp())
            .bind(GenericService::get_timestamp())
            .bind(&request.app_ipaddress)
            .fetch_one(&mut *trans).await {
                Ok(row) => row.get("autonid"),
                Err(e) => {
                    result.error = Some(format!("Failed to insert user_kyc: {}", e));
                    return result;
                }
            };
        let otp_generated_link: String = match sqlx::query(r#"
            INSERT INTO users 
            (web_cif_id, email, handphone, activate_code, password, register_date,
            disable_login, otp_generated_link, otp_generated_link_date, picture, google_id, client_category)
            VALUES
            ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
            RETURNING otp_generated_link;"#)
            .bind(auto_nid)
            .bind(&request.email)
            .bind(&request.mobile_phone)
            .bind(GenericService::random_string(20))
            .bind(&enc_password)
            .bind(GenericService::get_timestamp())
            .bind(true)
            .bind(GenericService::random_string(70))
            .bind(GenericService::get_timestamp())
            .bind("") // picture
            .bind("") // sub
            .bind(&request.client_category)
            .fetch_one(&mut *trans).await {
                Ok(row) => row.get("otp_generated_link"),
                Err(e) => {
                    result.error = Some(format!("Failed to insert users: {}", e));
                    return result;
                }
            };

        // INSERT ke user_request
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO user_request (web_cif_nid, referal)
            VALUES ($1, $2);
            "#
        )
        .bind(auto_nid)
        .bind(&request.referal)
        .execute(&mut *trans)
        .await
        {
            result.error = Some(format!("Failed to insert user_request: {}", e));
            return result;
        }

        // Commit transaksi
        if let Err(e) = trans.commit().await {
            result.error = Some(format!("Failed to commit transaction: {}", e));
            return result;
        }

        let mut mail_data = HashMap::new();
        mail_data.insert("FirstName".to_string(), request.full_name);
        mail_data.insert("ActivationURL".to_string(), Some(format!("{}/{}", front_url, otp_generated_link)));
        mail_data.insert("CompanyName".to_string(), Some("PT. MICRO PIRANTI COMPUTER".to_string()));
        mail_data.insert("subject".to_string(), Some("Verifikasi Akun Anda".to_string()));
        mail_data.insert("email".to_string(), request.email);

        let mail_result : ActionResult<String, String> = MailService::send(mail_data, "activation").await;
        
        if let Some(e) = mail_result.error {
            println!("❌ Mail Error: {}", e);
        }

        result.result = true;
        result.message = "User registered successfully".into();

        return result;
    }

    pub async fn activation(activation_url: String) -> ActionResult<String, String> {
        let mut result = ActionResult::default();

        let connection = CONNECTION.get().expect("DB_POOL not initialized");

        let mut trans = match connection.begin().await {
            Ok(trans) => trans,
            Err(e) => {
                result.error = Some(format!("Failed to start transaction: {}", e));
                return result;
            }
        };

        let query_result = match sqlx::query(r#"SELECT web_cif_id, activate_time, count_resend_activation 
                    FROM users 
                    WHERE otp_generated_link = $1;"#)
            .bind(&activation_url)
            .fetch_optional(&mut *trans).await {
                Ok(row) => row,
                Err(e) => {
                    result.error = Some(format!("Failed to insert users: {}", e));
                    return result;
                }
            
        };

        let (web_cif_id, activate_time, count_resend_activation): (Option<i32>, Option<chrono::NaiveDateTime>, Option<i32>) = match query_result {
            Some(row) => (row.get("web_cif_id"), row.get("activate_time"), row.get("count_resend_activation")),
            None => {
                result.message = "Invalid activation link".into();
                return result;
            }
        };

        if let Some(count_resend_activation) = count_resend_activation {
            if count_resend_activation > 0 {
                result.message = format!("You have already activated your account at {}", activate_time.unwrap_or_default().to_string());
                return result;
            }
        }

        if let Err(e) = sqlx::query(r#"UPDATE users
            SET count_resend_activation = count_resend_activation + 1, activate_time = $2, disable_login = $3
            WHERE web_cif_id = $1"#)
            .bind(web_cif_id)
            .bind(GenericService::get_timestamp())
            .bind(false)
            .execute(&mut *trans).await {
                let _ = trans.rollback().await;
                result.error = Some(format!("Failed to insert users: {}", e));
                return result;
            }

        if let Err(e) = trans.commit().await {
            result.error = Some(format!("Failed to commit transaction: {}", e));
            return result;
        }

        result.result = true;
        result.message = "Account activated successfully".into();
        return result;
    }

    pub async fn check_session(session: Claims, token: String, cookies: String, delete: bool, update: bool, exist: bool, app_name: &str) -> ActionResult<Claims, String> {
        let mut result: ActionResult<Claims, String> = ActionResult::default();

        let connection = CONNECTION.get().expect("DB_POOL not initialized");
        let active_token = if cookies.is_empty() { token.clone() } else { cookies.clone() };

        let mut trans = match connection.begin().await {
            Ok(t) => t,
            Err(e) => {
                result.error = Some(format!("Database error: {}", e));
                return result;
            }            
        };

        if exist {
            println!("Check Session");
            let row_count: i64 = match sqlx::query(r#"SELECT COUNT(*) as count FROM cookies WHERE user_nid = $1 AND token_cookie = $2"#)
                .bind(session.usernid)
                .bind(&active_token)
                .fetch_one(&mut *trans)
                .await {
                    Ok(row) => row.get("count"),
                    Err(e) => {
                        println!("❌ Check Session Error: {}", e);
                        0i64
                    }
                };

            if row_count == 0 {
                result.error = Some("Session has expired".to_string());
                return result;
            }

            if update {
                println!("Update Session 1");
                if let Err(e) = sqlx::query(r#"UPDATE cookies SET last_update = $1 WHERE user_nid = $2 AND token_cookie = $3"#)
                    .bind(GenericService::get_timestamp())
                    .bind(session.usernid)
                    .bind(&active_token)
                    .execute(&mut *trans)
                    .await {
                        result.error = Some(format!("Failed to update cookies: {}", e));
                        return result;
                    };
                result.result = true;
            }
            result.result = true;
        } else if delete {
            println!("Delete Session");
            if let Err(e) = sqlx::query(r#"DELETE FROM cookies WHERE user_nid = $1 AND token_cookie = $2"#)
                .bind(session.usernid)
                .bind(&active_token)
                .execute(&mut *trans)
                .await {
                    result.error = Some(format!("Failed to delete cookies: {}", e));
                    return result;
                };
            result.result = true;
        } else {
            let mut user_session: Claims = session.clone();
            if !cookies.is_empty() {
                println!("Update Session 2: {}", cookies);
                match  sqlx::query(r#"UPDATE cookies SET token_cookie = $1, last_update = $3 WHERE user_nid = $2"#)
                    .bind(&active_token)
                    .bind(session.usernid)
                    .bind(GenericService::get_timestamp())
                    .execute(&mut *trans)
                    .await {
                        Ok(row) => {
                            if row.rows_affected() == 0 {
                                result.error = Some("Session has expired".to_string());
                            } else {
                                result.result = true;
                                result.message = "Session updated successfully".to_string();
                                result.data = Some(user_session);
                            }
                        },
                        Err(e) => {
                            result.error = Some(format!("Failed to update cookies: {}", e));
                            return result;
                        }
                    };
            } else {
                println!("Check Session 2");
                let row_option = match sqlx::query(r#"SELECT token_cookie, last_update FROM cookies WHERE user_nid = $1"#)
                    .bind(session.usernid)
                    // .bind(&active_token)
                    .fetch_optional(&mut *trans)
                    .await {
                        Ok(row) => row,
                        Err(e) => {
                            result.error = Some(format!("Failed to fetch cookies: {}", e));
                            return result;
                        }
                    };

                    println!("{:?}", row_option);
                if let Some(row) = row_option {
                    println!("Check Session 3");
                    let last_update: Option<chrono::NaiveDateTime> = row.get::<chrono::NaiveDateTime, _>("last_update").format("%Y-%m-%d %H:%M:%S").to_string().parse().ok();
                    let user_token: Option<String> = row.get("token_cookie");
                    if let Ok(decode_session) = validate_jwt(&user_token.unwrap_or_default()) {
                        user_session = decode_session;
                        result.data = Some(user_session);
                    }

                    println!("Lastupdate: {}, now: {}", last_update.unwrap_or_default().to_string(), GenericService::get_timestamp().to_string());
                    
                    let expired_date: chrono::NaiveDateTime = chrono::NaiveDateTime::parse_from_str(
                        last_update.unwrap_or_default().to_string().as_str(), 
                        "%Y-%m-%d %H:%M:%S"
                    ).unwrap_or_else(|_| GenericService::get_timestamp()); // ⏳ Set exp untuk validasi JWT

                    if expired_date > GenericService::get_timestamp() {
                        result.message = format!(
                            "This user ({}) with IP:{} is already logged in from another browser/machine (LastUpdate: {}), are you sure you want to kick this logged in user?",
                            session.email,
                            session.ip_address.clone().expect("IP address not found"),
                            last_update.unwrap_or_default().to_string()
                        );
                        
                        result.data = Some(session);
                        
                        // nanti ada buat multiple session sama nendang
                        return result;
                    } else {
                        println!("Update cookies 3");
                        if let Err(e) = sqlx::query(r#"UPDATE cookies SET last_update = $1, token_cookie = $3 WHERE user_nid = $2 AND token_cookie = $3"#)
                            .bind(GenericService::get_timestamp())
                            .bind(session.usernid)
                            .bind(&active_token)
                            .execute(&mut *trans)
                            .await {
                                result.error = Some(format!("Failed to update cookies: {}", e));
                                return result;
                            };
                        result.result = true;
                    }
                } else {
                    println!("Insert cookies");
                    if let Err(e) = sqlx::query(r#"INSERT INTO cookies (user_nid, token_cookie, app_computer_name, app_ip_address, last_update, app_name) 
                                VALUES ($1, $2, $3, $4, $5, $6)"#)
                        .bind(session.usernid)
                        .bind(&active_token)
                        .bind(session.comp_name.clone().unwrap_or_default())
                        .bind(session.ip_address.clone().unwrap_or_default())
                        .bind(GenericService::get_timestamp())
                        .bind(app_name)
                        .execute(&mut *trans)
                        .await {
                            result.error = Some(format!("Failed to insert cookies: {}", e));
                            return result;
                        };
                    result.result = true;
                    user_session.app_name = Some(app_name.to_string());
                    result.message = "Login successfully".to_string();
                    result.data = Some(user_session);
                }
            }
        }

        if let Err(e) = trans.commit().await {
            result.error = Some(format!("Failed to commit transaction: {}", e));
            return result;
        };

        return result;
    }
}