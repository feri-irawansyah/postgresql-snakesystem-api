use actix_web::{get, web, HttpResponse, Responder, Scope};

use crate::{middleware::model::{ActionResult, HeaderParams, ResultList, TableDataParams}, services::{data_service::DataService, generic_service::GenericService}};


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

    let cache_key = GenericService::make_cache_key(&params.tablename, &params);

    let cached_data: &Result<ResultList, Box<dyn std::error::Error>> = &DataService::get_cache_data(&cache_key).await;

    if cached_data.is_ok() && cached_data.as_ref().unwrap().rows.len() > 0 {
        println!("Cache Hit");
        return HttpResponse::Ok().json(serde_json::json!({
            "total": cached_data.as_ref().unwrap().total,
            "totalNotFiltered": cached_data.as_ref().unwrap().total_not_filtered,
            "rows": cached_data.as_ref().unwrap().rows
        }));
    } else {
        println!("Query DB");
        let data: Result<ResultList, Box<dyn std::error::Error>> = DataService::get_table_data(params.into_inner()).await;

        match data {
            Ok(response) => {

                let saved_cached = DataService::set_cache_data(&cache_key, &response, 86400).await;

                if saved_cached.is_err() {
                    return HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": saved_cached.unwrap_err().to_string()})
                    );
                } else {
                    println!("{}", saved_cached.unwrap());
                }

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
}