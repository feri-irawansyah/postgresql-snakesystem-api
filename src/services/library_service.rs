use actix_web::HttpRequest;
use sqlx::Row;
use crate::{middleware::model::{ActionResult, NewNoteRequest, NewPortfolioRequest, NewSkillRequest, Notes, Portfolio, Skill, SkillSummary, UpdateNoteRequest, UpdatePortfolioRequest, UpdateSkillRequest}, services::generic_service::GenericService, CONNECTION};

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
        let mut result: ActionResult<Notes, String> = ActionResult::default();

        let query_result = sqlx::query(r#"SELECT * FROM notes WHERE slug = $1"#)
        .bind(&slug)
        .fetch_one(connection)
        .await;

        match query_result {
            Ok(row) => {
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
                    last_update: row.try_get::<chrono::DateTime<chrono::Utc>, _>("last_update").unwrap_or_else(|_| chrono::Utc::now()),
                    hashtag: row.try_get::<Vec<String>, _>("hashtag").unwrap_or_default(), 
                    description: row.try_get::<String, _>("description").unwrap_or_default()
                 });
            }
            Err(e) => {
                result.message = format!("Invalid request: {}", e);
                println!("❌ Login Error: {}", e);
            }
        }

        result
    }

    pub async fn create_skill(request: NewSkillRequest) -> ActionResult<String, String> {
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
            r#"INSERT INTO skills (title, description, url_docs, image_src, progress, star) 
            VALUES ($1, $2, $3, $4, $5, $6)"#)
            .bind(&request.title)
            .bind(&request.description)
            .bind(&request.url_docs)
            .bind(&request.image_src)
            .bind(&request.progress)
            .bind(&request.star)
            .execute(&mut *trans)
        .await {
            result.error = Some(format!("Failed to insert skills: {}", e));
            return result;
        };

        if let Err(e) = trans.commit().await {
            result.error = Some(format!("Failed to commit transaction: {}", e));
            return result;
        };

        result.result = true;
        result.message = "Skill created successfully".to_string();

        return result;
    }

    pub async fn update_skill(request: UpdateSkillRequest) -> ActionResult<String, String> {
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
            r#"UPDATE skills SET title = $1, description = $2, url_docs = $3, image_src = $4, progress = $5, star = $6 WHERE skill_id = $7"#)
            .bind(&request.title)
            .bind(&request.description)
            .bind(&request.url_docs)
            .bind(&request.image_src)
            .bind(&request.progress)
            .bind(&request.star)
            .bind(&request.skill_id)
            .execute(&mut *trans)
        .await {
            result.error = Some(format!("Failed to update skills: {}", e));
            return result;
        };

        if let Err(e) = trans.commit().await {
            result.error = Some(format!("Failed to commit transaction: {}", e));
            return result;
        };

        result.result = true;
        result.message = "Skill updated successfully".to_string();

        return result;
    }

    pub async fn get_skill(skill_id: i32) -> ActionResult<Skill, String> {

        let connection: &sqlx::PgPool = CONNECTION.get().unwrap();
        let mut result: ActionResult<Skill, String> = ActionResult::default();

        let query_result = sqlx::query(r#"SELECT * FROM skills WHERE skill_id = $1"#)
        .bind(&skill_id)
        .fetch_one(connection)
        .await;

        match query_result {
            Ok(row) => {
                if Some(row.try_get::<i32, _>("skill_id").unwrap_or(0)) == None {
                    result.error = Some("Skill not found".to_string());
                    return result;
                }

                result.result = true;
                result.data = Some(Skill { 
                    skill_id: row.try_get::<i32, _>("skill_id").unwrap_or(0), 
                    description: row.try_get::<String, _>("description").unwrap_or_default(), 
                    title: row.try_get::<String, _>("title").unwrap_or_default(), 
                    url_docs: row.try_get::<String, _>("url_docs").unwrap_or_default(), 
                    image_src: row.try_get::<String, _>("image_src").unwrap_or_default(), 
                    progress: row.try_get::<i32, _>("progress").unwrap_or(0), 
                    star: row.try_get::<i32, _>("star").unwrap_or(0), 
                    last_update: row.try_get::<chrono::DateTime<chrono::Utc>, _>("last_update").unwrap_or_else(|_| chrono::Utc::now())
                 });
            }
            Err(e) => {
                result.message = format!("Incorect skill");
                println!("❌ Login Error: {}", e);
            }
        }

        result
    }

    pub async fn create_portfolio(request: NewPortfolioRequest) -> ActionResult<String, String> {
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
            r#"INSERT INTO portfolio (title, description, url_docs, image_src, tech) 
            VALUES ($1, $2, $3, $4, $5)"#)
            .bind(&request.title)
            .bind(&request.description)
            .bind(&request.url_docs)
            .bind(&request.image_src)
            .bind(&request.tech)
            .execute(&mut *trans)
        .await {
            result.error = Some(format!("Failed to insert portfoluo: {}", e));
            return result;
        };

        if let Err(e) = trans.commit().await {
            result.error = Some(format!("Failed to commit transaction: {}", e));
            return result;
        };

        result.result = true;
        result.message = "Portfolio created successfully".to_string();

        return result;
    }

    pub async fn update_portfolio(request: UpdatePortfolioRequest) -> ActionResult<String, String> {
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
            r#"UPDATE portfolio SET title = $1, description = $2, url_docs = $3, image_src = $4, tech = $5 WHERE portfolio_id = $6"#)
            .bind(&request.title)
            .bind(&request.description)
            .bind(&request.url_docs)
            .bind(&request.image_src)
            .bind(&request.tech)
            .bind(&request.portfolio_id)
            .execute(&mut *trans)
        .await {
            result.error = Some(format!("Failed to update portfolio: {}", e));
            return result;
        };

        if let Err(e) = trans.commit().await {
            result.error = Some(format!("Failed to commit transaction: {}", e));
            return result;
        };

        result.result = true;
        result.message = "Portfolio updated successfully".to_string();

        return result;
    }

    pub async fn get_portfolio(portfolio_id: i32) -> ActionResult<Portfolio, String> {

        let connection: &sqlx::PgPool = CONNECTION.get().unwrap();
        let mut result: ActionResult<Portfolio, String> = ActionResult::default();

        let query_result = sqlx::query(r#"
            SELECT 
                p.portfolio_id,
                p.title,
                p.url_docs,
                p.image_src,
                p.description,
                json_agg(
                    json_build_object(
                        'url_docs', s.url_docs,
                        'title', s.title,
                        'image_src', s.image_src
                    )
                    ORDER BY array_position(p.tech, s.skill_id)
                ) AS tech,
                p.last_update
            FROM 
                portfolio p,
                LATERAL unnest(p.tech) AS tech_id
                JOIN skills s ON s.skill_id = tech_id
            GROUP BY 
                p.portfolio_id, p.title;
        "#)
        .bind(&portfolio_id)
        .fetch_one(connection)
        .await;

        match query_result {
            Ok(row) => {
                if Some(row.try_get::<i32, _>("portfolio_id").unwrap_or(0)) == None {
                    result.error = Some("Skill not found".to_string());
                    return result;
                }

                let tech_json: serde_json::Value = row.try_get("tech").unwrap_or(serde_json::Value::Null); 

                result.result = true;
                result.data = Some(Portfolio { 
                    portfolio_id: row.try_get::<i32, _>("portfolio_id").unwrap_or(0), 
                    description: row.try_get::<String, _>("description").unwrap_or_default(), 
                    title: row.try_get::<String, _>("title").unwrap_or_default(), 
                    url_docs: row.try_get::<String, _>("url_docs").unwrap_or_default(), 
                    image_src: row.try_get::<String, _>("image_src").unwrap_or_default(), 
                    tech: serde_json::from_value(tech_json).unwrap_or_else(|e| {
                        eprintln!("❌ Failed to parse tech JSON: {:?}", e);
                        vec![]
                    }),
                    last_update: row.try_get::<chrono::DateTime<chrono::Utc>, _>("last_update").unwrap_or_else(|_| chrono::Utc::now())
                 });
            }
            Err(e) => {
                result.message = format!("Incorect request: {}", e);
                println!("❌ Login Error: {}", e);
            }
        }

        result
    }
}