use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

mod printer;

#[derive(Serialize, Deserialize)]
struct PrintRequest {
    line1: String,
    line2: Option<String>,
    line3: Option<String>,
    line4: Option<String>,
    printer_name: Option<String>,
    label_size: Option<String>,
}

#[derive(Serialize)]
struct PrintResponse {
    success: bool,
    message: String,
    job_id: Option<String>,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    version: String,
}

async fn health_check() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn print_label(Json(payload): Json<PrintRequest>) -> Result<Json<PrintResponse>, StatusCode> {
    let lines = vec![
        Some(payload.line1),
        payload.line2,
        payload.line3,
        payload.line4,
    ];

    match printer::print_lines(&lines, payload.printer_name.as_deref(), payload.label_size.as_deref()).await {
        Ok(job_id) => Ok(Json(PrintResponse {
            success: true,
            message: "Print job submitted successfully".to_string(),
            job_id: Some(job_id),
        })),
        Err(e) => {
            eprintln!("Print error: {}", e);
            Ok(Json(PrintResponse {
                success: false,
                message: format!("Print failed: {}", e),
                job_id: None,
            }))
        }
    }
}

async fn list_printers() -> Result<Json<Vec<String>>, StatusCode> {
    match printer::list_printers().await {
        Ok(printers) => Ok(Json(printers)),
        Err(e) => {
            eprintln!("Failed to list printers: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/print", post(print_label))
        .route("/printers", get(list_printers))
        .fallback_service(ServeDir::new("static"))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    println!("Label server running on http://0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
