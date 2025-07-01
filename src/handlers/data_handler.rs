use actix_web::{get, web, HttpResponse, Responder, Scope};

use crate::{middleware::model::{ResultList, TableDataParams}, services::data_service::DataService};


pub fn data_scope() -> Scope {
    
    web::scope("/data")
        .service(get_table)
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