use actix_web::HttpRequest;
use sqlx::Row;
use crate::{middleware::model::{ActionResult, NewNoteRequest, Notes, UpdateNoteRequest}, services::generic_service::GenericService, CONNECTION};

pub struct LibraryService;

impl LibraryService {

    pub async fn create_library(req: HttpRequest, request: NewNoteRequest) -> ActionResult<String, String> {
        let mut result = ActionResult::default();
        let connection: &sqlx::PgPool = CONNECTION.get().unwrap();

        let mut trans = match connection.begin().await {
            Ok(t) => t,
            Err(e) => {
                result.error = Some(format!("Database error: {}", e));
                return result;
            }            
        };

        if let Err(e) = sqlx::query(
            r#"INSERT INTO notes (category, title, slug, content, ip_address, description, hashtag) 
            VALUES ($1, $2, $3, $4, $5, $6, $7)"#)
            .bind(&request.category)
            .bind(&request.title)
            .bind(&request.slug)
            .bind(&request.content)
            .bind(GenericService::get_ip_address(&req))
            .bind(&request.description)
            .bind(&request.hashtag)
            .execute(&mut *trans)
        .await {
            result.error = Some(format!("Failed to insert notes: {}", e));
            return result;
        };

        if let Err(e) = trans.commit().await {
            result.error = Some(format!("Failed to commit transaction: {}", e));
            return result;
        };

        result.result = true;
        result.message = "Note created successfully".to_string();

        return result;
    }

    pub async fn update_library(req: HttpRequest, request: UpdateNoteRequest) -> ActionResult<String, String> {
        let mut result = ActionResult::default();
        let connection: &sqlx::PgPool = CONNECTION.get().unwrap();

        let mut trans = match connection.begin().await {
            Ok(t) => t,
            Err(e) => {
                result.error = Some(format!("Database error: {}", e));
                return result;
            }            
        };

        if let Err(e) = sqlx::query(
            r#"UPDATE notes SET category = $1, title = $2, slug = $3, content = $4, ip_address = $5, description = $6, hashtag = $7 WHERE notes_id = $8"#)
            .bind(&request.category)
            .bind(&request.title)
            .bind(&request.slug)
            .bind(&request.content)
            .bind(GenericService::get_ip_address(&req))
            .bind(&request.description)
            .bind(&request.hashtag)
            .bind(&request.notes_id)
            .execute(&mut *trans)
        .await {
            result.error = Some(format!("Failed to update notes: {}", e));
            return result;
        };

        if let Err(e) = trans.commit().await {
            result.error = Some(format!("Failed to commit transaction: {}", e));
            return result;
        };

        result.result = true;
        result.message = "Note created successfully".to_string();

        return result;
    }

    pub async fn get_library(slug: String) -> ActionResult<Notes, String> {

        let connection: &sqlx::PgPool = CONNECTION.get().unwrap();
        let mut result = ActionResult::default();

        let query_result = sqlx::query(r#"SELECT * FROM notes WHERE slug = $1"#)
        .bind(&slug)
        .fetch_one(connection)
        .await;

        match query_result {
            Ok(row) => {
                println!("Row: {:?}", row);
                if Some(row.try_get::<i32, _>("notes_id").unwrap_or(0)) == None {
                    result.error = Some("Notes not found".to_string());
                    return result;
                }

                result.result = true;
                result.data = Some(Notes { 
                    notes_id: row.try_get::<i32, _>("notes_id").unwrap_or(0), 
                    category: row.try_get::<String, _>("category").unwrap_or_default(), 
                    title: row.try_get::<String, _>("title").unwrap_or_default(), 
                    slug: row.try_get::<String, _>("slug").unwrap_or_default(), 
                    content: row.try_get::<String, _>("content").unwrap_or_default(), 
                    ip_address: row.try_get::<String, _>("ip_address").unwrap_or_default(), 
                    last_update: row.try_get::<chrono::DateTime<chrono::Utc>, _>("last_update").unwrap_or_else(|_| chrono::Utc::now())
                 });
            }
            Err(e) => {
                result.message = format!("Incorrect email or password");
                println!("❌ Login Error: {}", e);
            }
        }

        result
    }
}