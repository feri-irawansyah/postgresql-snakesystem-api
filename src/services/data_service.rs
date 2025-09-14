use std::collections::HashMap;
use std::fmt::Write;
use redis::Commands;
use sqlx::Row;
use sqlx::Column;
use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::Asia::Jakarta;
use crate::middleware::model::ActionResult;
use crate::REDIS_CLIENT;
use crate::{middleware::model::{QueryClass, ResultList, TableDataParams}, CONNECTION};

pub struct DataService;

impl DataService {

    pub async fn get_header(tablename: String) -> ActionResult<Vec<serde_json::Value>, String> {
        let mut result: ActionResult<Vec<serde_json::Value>, String> = ActionResult::default();
        let connection: &sqlx::PgPool = CONNECTION.get().unwrap();

        // Panggil PostgreSQL function dan ambil JSON-nya
        let query = r#"SELECT create_table_object($1) AS result"#;

        match sqlx::query(query)
            .bind(&tablename)
            .fetch_one(&*connection)
            .await
        {
            Ok(row) => {
                let raw_json: serde_json::Value = row.try_get::<serde_json::Value, _>("result").unwrap_or(serde_json::Value::Null);

                match serde_json::from_value(raw_json) {
                    Ok(parsed_json) => {
                        result.result = true;
                        result.data = Some(parsed_json);
                        result.message = "Data retrieved successfully".to_string();
                    }
                    Err(e) => {
                        result.message = "Failed to parse JSON".to_string();
                        result.error = Some(e.to_string());
                    }
                }
            }
            Err(e) => {
                result.message = "Query failed".to_string();
                result.error = Some(e.to_string());
            }
        }

        result
    }

    pub async fn get_cache_data(cache_key: &str) -> Result<ResultList, Box<dyn std::error::Error>> {

        let mut result = ResultList {
            total_not_filtered: 0,
            total: 0,
            rows: vec![],
        };

        let connection = REDIS_CLIENT.get().expect("Redis not initialized").clone();

        let data: Option<String> = connection.clone().get(cache_key)?;

        if data.is_some() {
            result = serde_json::from_str(data.unwrap().as_str()).unwrap();
        }
        
        Ok(result)
    }

    pub async fn set_cache_data(cache_key: &str, data: &ResultList, ttl_secs: usize) -> Result<String, redis::RedisError> {
        let mut connection = REDIS_CLIENT.get().expect("Redis not initialized").clone();

        let serialized = serde_json::to_string(data).unwrap();

        // kasih tipe generic return jadi ()
        if let Err(e) = connection
            .set_ex::<&str, String, ()>(cache_key, serialized, ttl_secs.try_into().unwrap())
             {
                return Err(e);
            };

        Ok("Cache saved".to_string())
    }

    pub async fn get_table_data(allparams: TableDataParams) -> Result<ResultList, Box<dyn std::error::Error>> {
        let mut result = ResultList {
            total_not_filtered: 0,
            total: 0,
            rows: vec![],
        };

        let connection: &sqlx::PgPool = CONNECTION.get().unwrap();

        let query = Self::get_query_table(&allparams, false);

        if !allparams.tablename.is_empty() {
            let rows = sqlx::query(query.query_total_all.as_str())
                    .persistent(false)
                    .fetch_optional(connection).await?;
            if let Some(r) = rows {
                result.total_not_filtered = r.try_get::<i64, _>(0).unwrap_or(0);
            }
        }

        // Hitung total data yang sesuai filter
        if let Some(filter) = &allparams.filter {
            if filter != "{filter:undefined}" {
                let row = sqlx::query(query.query_total_with_filter.as_str())
                .persistent(false)
                .fetch_optional(connection).await?;
                // let row: Option<Row> = client.query(query.query_total_with_filter.clone(), &[]).await?.into_row().await?;
                if let Some(r) = row {
                    result.total = r.try_get::<i64, _>(0).unwrap_or(0);
                }
            }
        } else {
            result.total = result.total_not_filtered;
        }

        let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(query.query.as_str())
        .persistent(false)
        .fetch_all(connection).await?;

        let json_rows: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                let map = Self::row_to_json(row); // dari function yang kamu buat
                serde_json::Value::Object(map)          // bungkus jadi Value::Object
            })
            .collect();

        result.rows = json_rows;

        Ok(result)
    }

    fn get_query_table(allparams: &TableDataParams, bypass_skip: bool) -> QueryClass {
        let mut result = QueryClass {
            query: String::new(),
            query_total_all: String::new(),
            query_total_with_filter: String::new(),
        };
    
        if allparams.limit == 0 {
            return result;
        }
    
        let tablename = format!("{}", allparams.tablename);
        let mut query_total_all = format!("SELECT count({}) as total FROM {}", allparams.nidkey.as_deref().unwrap_or_else(|| "AutoNID"), tablename);
        let mut q_and_where = String::from(" WHERE 1=1 ");
        let mut q_order_by = String::new();
        let mut q_skip_row = String::new();
        let mut q_and_where_for_total_with_filter = String::from(" WHERE 1=1 ");
    
        // Gunakan `nidkey` sebagai primary key jika tersedia
        let q_primary_key = allparams.nidkey.as_deref().unwrap_or_else(|| "AutoNID");
        let q_primary_key_order = q_primary_key;
    
        // Tambahkan filter jika ada
        if let Some(filter) = &allparams.filter {
            if filter != "{filter:undefined}" {
                if let Ok(filter_name) = serde_json::from_str::<HashMap<String, String>>(filter) {
                    if !filter_name.is_empty() {
                        let q_and_where_result = Self::get_query_table_where(q_and_where, filter_name);
    
                        q_and_where = q_and_where_result.clone();
                        q_and_where_for_total_with_filter = q_and_where_result;
                    }
                }
            }
        }
    
        query_total_all.push_str(&q_and_where);
    
        let query_total_with_filter = format!(
            "SELECT count({}) as totalWithFilter FROM {} {}",
            q_primary_key, tablename, q_and_where_for_total_with_filter
        );
    
        result.query_total_with_filter = query_total_with_filter;
    
        // Sorting & pagination
        if !bypass_skip {
            if let Some(sort) = &allparams.sort {
                if let Some(order) = &allparams.order {
                    let _ = write!(q_order_by, " ORDER BY {} {}", sort, order);
                }
            } else {
                let _ = write!(q_order_by, " ORDER BY {} DESC", q_primary_key_order);
            }
    
            let _ = write!(
                q_skip_row,
                " OFFSET {} ROWS FETCH NEXT {} ROWS ONLY",
                allparams.offset, allparams.limit
            );
        }
    
        // Query utama
        result.query = format!(
            "SELECT * FROM {} {} {} {}",
            tablename, q_and_where, q_order_by, q_skip_row
        );
    
        result.query_total_all = query_total_all;
        result
    }

    fn get_query_table_where(mut fquery: String, filter_name: HashMap<String, String>) -> String {
        for (key, value) in filter_name {
            if let Ok(temp_date) = chrono::NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
                if key.ends_with("date") {
                    let next_date = temp_date.succ_opt().unwrap_or(temp_date);
                    let _ = write!(
                        fquery,
                        " AND {} BETWEEN '{}' AND '{}'",
                        key, value, next_date
                    );
                } else {
                    let _ = write!(fquery, " AND {} = '{}'", key, value);
                }
            } else if key.ends_with("time") {
                let dates: Vec<&str> = value.split("to").collect();
                if dates.len() == 2 {
                    let _ = write!(
                        fquery,
                        " AND {} BETWEEN '{} 00:00:00' AND '{} 23:59:59'",
                        key, dates[0], dates[1]
                    );
                }
            } else if key.starts_with('_') || key.ends_with("nid") || key.ends_with("id") {
                let _ = write!(fquery, " AND {} = '{}'", key, value);
            } else {
                let _ = write!(fquery, " AND {} LIKE '%{}%'", key, value);
            }
        }
    
        fquery
    }

    pub fn pg_value_to_json(row: &sqlx::postgres::PgRow, col: &str) -> serde_json::Value {
        let type_name = row.column(col).type_info().to_string();

        match type_name.as_str() {
            "INT2" => {
                let val: Result<i16, _> = row.try_get(col);
                val.ok()
                    .map(|v| serde_json::Value::Number(v.into()))
                    .unwrap_or(serde_json::Value::Null)
            }
            "INT4" => {
                let val: Result<i32, _> = row.try_get(col);
                val.ok()
                    .map(|v| serde_json::Value::Number(v.into()))
                    .unwrap_or(serde_json::Value::Null)
            }
            "INT8" => {
                let val: Result<i64, _> = row.try_get(col);
                val.ok()
                    .map(|v| serde_json::Value::Number(v.into()))
                    .unwrap_or(serde_json::Value::Null)
            }
            "FLOAT4" => {
                let val: Result<f32, _> = row.try_get(col);
                val.ok()
                    .and_then(|v| serde_json::Number::from_f64(v as f64))
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            }
            "FLOAT8" => {
                let val: Result<f64, _> = row.try_get(col);
                val.ok()
                    .and_then(serde_json::Number::from_f64)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            }
            "BOOL" => {
                let val: Result<bool, _> = row.try_get(col);
                val.ok().map(serde_json::Value::Bool).unwrap_or(serde_json::Value::Null)
            }
            "TEXT" | "VARCHAR" | "UUID" => {
                let val: Result<String, _> = row.try_get(col);
                val.ok().map(serde_json::Value::String).unwrap_or(serde_json::Value::Null)
            }
            "TEXT[]" | "_TEXT" | "VARCHAR[]" | "_VARCHAR" => {
                let val: Result<Vec<String>, _> = row.try_get(col);
                val.ok()
                    .map(|v| serde_json::Value::Array(v.into_iter().map(serde_json::Value::String).collect()))
                    .unwrap_or(serde_json::Value::Null)
            }
            "INT2[]" | "_INT2" | "INT4[]" | "_INT4" | "INT8[]" | "_INT8" => {
                let val: Result<Vec<i32>, _> = row.try_get(col);
                val.ok()
                    .map(|v| serde_json::Value::Array(v.into_iter().map(|arg0: i32| serde_json::Value::Number(arg0.into())).collect()))
                    .unwrap_or(serde_json::Value::Array(Vec::new()))
            }
            "TIMESTAMP" | "TIMESTAMPTZ" | "DATE" => {
                let val: Result<NaiveDateTime, _> = row.try_get(col);

                val.ok()
                    .map(|dt| {
                        let utc: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc);
                        let jakarta_time = utc.with_timezone(&Jakarta);
                        serde_json::Value::String(jakarta_time.to_rfc3339())
                    })
                    .unwrap_or(serde_json::Value::Null)
            }
            _ => {
                // fallback ke string kalau tidak dikenali
                let val: Result<String, _> = row.try_get(col);
                val.ok().map(serde_json::Value::String).unwrap_or(serde_json::Value::Null)
            }
        }
    }

    pub fn row_to_json(row: &sqlx::postgres::PgRow) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();

        for column in row.columns() {
            let col_name = column.name();
            let value = Self::pg_value_to_json(row, col_name);
            map.insert(col_name.to_string(), value);
        }

        map
    }
    
    
}