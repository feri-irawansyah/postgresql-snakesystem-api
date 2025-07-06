use actix_web::{get, web, HttpResponse, Responder, Scope};

use crate::{middleware::model::{ActionResult, HeaderParams, ResultList, TableDataParams}, services::data_service::DataService};


pub fn data_scope() -> Scope {
    
    web::scope("/data").configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_table)
        .service(get_header);
}

#[get("/header")]
pub async fn get_header(params: web::Query<HeaderParams>) -> impl Responder {

    let result: ActionResult<Vec<serde_json::Value>, String> = DataService::get_header(params.into_inner().tablename).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": response.error}))
        }, 
        response if response.result => {
            HttpResponse::Ok().json(serde_json::json!({"data": response.data}))
        }, 
        response => {
            HttpResponse::BadRequest().json(serde_json::json!({"error": response.message}))
        }
    }
}

#[get("/table")]
async fn get_table(params: web::Query<TableDataParams>) -> impl Responder {

    let data: Result<ResultList, Box<dyn std::error::Error>> = DataService::get_table_data(params.into_inner()).await;

    match data {
        Ok(response) => {
            return HttpResponse::Ok().json(serde_json::json!({
                "total": response.total,
                "totalNotFiltered": response.total_not_filtered,
                "rows": response.rows
            }));
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": e.to_string()})
            );
        },
        
    }
}