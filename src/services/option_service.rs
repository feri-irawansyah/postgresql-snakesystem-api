use serde_json::{Number, Value};
use sqlx::Row;
use sqlx::Column;

use crate::{middleware::model::ActionResult, CONNECTION};

pub struct OptionService;

impl OptionService {
    pub async fn get_options(code: &str, keyword: Option<&str>) -> ActionResult<Value, String> {
        let mut result = ActionResult::default();
        let connection = CONNECTION.get().unwrap();

        let config_row: sqlx::postgres::PgRow = match sqlx::query(
            "SELECT code, table_name, display_cols, mode, searchable_col, condition 
             FROM lookup_config WHERE code = $1"
        )
        .bind(code)
        .fetch_optional(connection)
        .await {
            Ok(Some(row)) => row,
            Ok(None) => {
                result.error = Some("Config not found".into());
                return result;
            },
            Err(e) => {
                result.error = Some(format!("Database error: {}", e));
                return result;
            }
        };


        let table: String = config_row.get("table_name");
        let cols: Vec<String> = config_row.get("display_cols");
        let mode: String = config_row.get("mode");
        let search_col: String = config_row.try_get("searchable_col").unwrap_or_else(|_| "name".to_string());
        let col_str = cols.join(", ");
        let condition: Option<String> = Some(config_row.try_get("condition").unwrap_or_else(|_| "".to_string()));

        let where_clause = match condition {
            Some(c) => format!(" WHERE {} = '{}'", search_col, c),
            None => "".to_string()
        };

        let rows = if mode == "autocomplete" && keyword.is_some() {
            let sql = format!(
                "SELECT {} FROM {} WHERE {} ILIKE $1 OR {}::TEXT ILIKE $1 LIMIT 20",
                col_str,
                table,
                search_col,
                cols[0] // asumsi cols[0] = ID column seperti country_id
            );

            sqlx::query(&sql)
                .bind(format!("%{}%", keyword.unwrap()))
                .fetch_all(connection)
                .await
        } else {
            let sql = format!("SELECT {} FROM {} {} LIMIT 100", col_str, table, where_clause);
            println!("SQL: {}", sql);
            sqlx::query(&sql)
                .fetch_all(connection)
                .await
        };

        match rows {
            Ok(data) => {
                let json: Vec<_> = data.into_iter().map(|row| {
                    let mut map: serde_json::Map<String, Value> = serde_json::Map::new();
                    for col in &cols {
                        let value = Self::pg_value_to_json(&row, col);
                        map.insert(col.clone(), value);
                    }
                    Value::Object(map)
                }).collect();

                result.result = true;
                result.data = Some(Value::Array(json));
            },
            Err(e) => {
                result.error = Some(format!("Failed to fetch data: {}", e));
            }
        }

        result
    }

    pub async fn get_options_city(keyword: Option<&str>) -> ActionResult<Value, String> {
        let mut result = ActionResult::default();
        let connection = CONNECTION.get().unwrap();

        let keyword = keyword.unwrap_or("").trim();

        // Deteksi apakah keyword berupa angka
        let is_number = keyword.parse::<i64>().is_ok();

        let sql = if is_number {
            r#"
                SELECT province_city_id AS id, sbr_province_name AS province, sbr_city_name AS city
                FROM province_city
                WHERE province_city_id = $1
                LIMIT 1
            "#
        } else {
            r#"
                SELECT province_city_id AS id, sbr_province_name AS province, sbr_city_name AS city
                FROM province_city
                WHERE sbr_province_name IS NOT NULL 
                AND sbr_city_name IS NOT NULL
                AND sbr_city_name ILIKE $1
                LIMIT 20
            "#
        };

        let query_result = if is_number {
            sqlx::query(sql)
                .bind(keyword.parse::<i64>().unwrap())
                .fetch_all(connection)
                .await
        } else {
            sqlx::query(sql)
                .bind(format!("%{}%", keyword))
                .fetch_all(connection)
                .await
        };

        match query_result {
            Ok(data) => {
                let json: Vec<_> = data.into_iter().map(|row| {
                    let mut map: serde_json::Map<String, Value> = serde_json::Map::new();
                    for col in &["id", "province", "city"] {
                        let value = Self::pg_value_to_json(&row, col);
                        map.insert(col.to_string(), value);
                    }
                    Value::Object(map)
                }).collect();

                result.result = true;
                result.data = Some(Value::Array(json));
            },
            Err(e) => {
                result.error = Some(format!("Failed to fetch data: {}", e));
            }
        }

        result
    }

    pub async fn get_question_npwp() -> ActionResult<Vec<serde_json::Value>, String> {
        let mut result = ActionResult::default();
        let mut question_npwp: Vec<serde_json::Value> = Vec::new(); 

        let data_array = vec![
            serde_json::json!({"option_key": 1, "option_value": "Memiliki NPWP"}),
            serde_json::json!({"option_key": 2, "option_value": "Tidak memiliki NPWP - Ikut pasangan"}),
            serde_json::json!({"option_key": 3, "option_value": "Tidak memiliki NPWP - Belum bekerja"}),
            serde_json::json!({"option_key": 4, "option_value": "Tidak memiliki NPWP - Alasan lainnya"}),
        ];

        question_npwp.extend(data_array.clone());

        result.result = true;
        result.data = Some(question_npwp.clone());

        return result;
    }

    pub fn pg_value_to_json(row: &sqlx::postgres::PgRow, col: &str) -> Value {
        let type_name = row.column(col).type_info().to_string();

        match type_name.as_str() {
            "INT2" => {
                let val: Result<i16, _> = row.try_get(col);
                val.ok()
                    .map(|v| Value::Number(v.into()))
                    .unwrap_or(Value::Null)
            }
            "INT4" => {
                let val: Result<i32, _> = row.try_get(col);
                val.ok()
                    .map(|v| Value::Number(v.into()))
                    .unwrap_or(Value::Null)
            }
            "INT8" => {
                let val: Result<i64, _> = row.try_get(col);
                val.ok()
                    .map(|v| Value::Number(v.into()))
                    .unwrap_or(Value::Null)
            }
            "FLOAT4" => {
                let val: Result<f32, _> = row.try_get(col);
                val.ok()
                    .and_then(|v| Number::from_f64(v as f64))
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            }
            "FLOAT8" => {
                let val: Result<f64, _> = row.try_get(col);
                val.ok()
                    .and_then(Number::from_f64)
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            }
            "BOOL" => {
                let val: Result<bool, _> = row.try_get(col);
                val.ok().map(Value::Bool).unwrap_or(Value::Null)
            }
            "TEXT" | "VARCHAR" | "UUID" | "TIMESTAMP" | "DATE" => {
                let val: Result<String, _> = row.try_get(col);
                val.ok().map(Value::String).unwrap_or(Value::Null)
            }
            _ => {
                // fallback ke string kalau tidak dikenali
                let val: Result<String, _> = row.try_get(col);
                val.ok().map(Value::String).unwrap_or(Value::Null)
            }
        }
    }

    pub fn row_to_json(row: &sqlx::postgres::PgRow) -> serde_json::Map<String, Value> {
        let mut map = serde_json::Map::new();

        for column in row.columns() {
            let col_name = column.name();
            let value = Self::pg_value_to_json(row, col_name);
            map.insert(col_name.to_string(), value);
        }

        map
    }

}
