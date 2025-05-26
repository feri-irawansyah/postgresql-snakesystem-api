use actix_web::{error, HttpRequest, HttpResponse, Responder};
use rand::{rng, Rng};
use crate::middleware::model::ActionResult;

pub struct GenericService;

impl GenericService {

    pub fn json_error_handler(err: error::JsonPayloadError, _req: &actix_web::HttpRequest) -> actix_web::Error {
        let error_message = format!("Json deserialize error: {}", err);

        let result = ActionResult::<String, _> {
            // <- Ubah dari ActionResult<()> ke ActionResult<String>
            result: false,
            message: "Invalid Request".to_string(),
            error: Some(error_message), // <- Sekarang cocok karena `data: Option<String>`
            data: None,
        };

        error::InternalError::from_response(err, HttpResponse::BadRequest().json(result)).into()
    }

    pub async fn not_found(req: HttpRequest) -> impl Responder {

        if req.path() == "/docs" {
            return HttpResponse::Found()
            .append_header(("Location", "/docs/index.html"))
            .finish()
        }

        HttpResponse::NotFound().json({
            serde_json::json!({
                "result": false,
                "message": "Not Found",
                "error": format!("Url '{}' not found. Please check the URL.", req.path())
            })
        })
    }

    pub fn random_string(length: usize) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rng();
    
        (0..length)
            .map(|_| {
                let idx = rng.random_range(0..CHARS.len());
                CHARS[idx] as char
            })
            .collect()
    }

    pub fn get_ip_address(req: &HttpRequest) -> String {
        req.headers()
            .get("X-Forwarded-For") // Jika pakai reverse proxy seperti Nginx
            .and_then(|ip| ip.to_str().ok())
            .map_or_else(
                || req.peer_addr()
                    .map(|addr| addr.ip().to_string())
                    .unwrap_or_else(|| "Unknown IP".to_string()),
                |ip| ip.to_string(),
            )
    }

    pub fn get_device_name(req: &HttpRequest) -> String {
        let test = req.headers()
            .get("X-Forwarded-Host")
            .and_then(|ua| ua.to_str().ok())
            .map_or_else(
                || "Unknown Device".to_string(),
                |ua| ua.to_string(),
            );

        return test
    }

    pub fn is_localhost_origin(req: &HttpRequest) -> bool {
        if let Some(origin) = req.headers().get("Origin") {
            if let Ok(origin_str) = origin.to_str() {
                return origin_str.starts_with("http://localhost");
            }
        }
        false
    }

    pub fn get_secret_key() -> [u8; 32] {
        let key_str = std::env::var("JWT_KEY").expect("JWT_KEY not set");
        let key_bytes = key_str.as_bytes();

        if key_bytes.len() != 32 {
            panic!("JWT_KEY must be exactly 32 bytes");
        }

        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes[..32]);
        key_array
    }

    pub fn slugify(title: &str) -> String {
        title
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() || c == '-' {
                    '-'
                } else {
                    '\0' // dibuang nanti
                }
            })
            .collect::<String>()
            .split('-') // hilangkan extra dash
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
    
    pub fn sanitize_filename(filename: &str) -> String {
        filename
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

}