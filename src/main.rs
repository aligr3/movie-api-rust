use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use reqwest::{Client, header::{HeaderMap, HeaderValue}};
use tower_http::cors::{CorsLayer, Any};
use std::fs;

mod movie;
use movie::Movie;

#[derive(Serialize)]
struct ApiInfo {
    message: String,
    endpoints: Vec<String>,
}

#[derive(Deserialize)]
struct PaginationParams {
    #[serde(default = "default_limit")]
    limit: i32,
    #[serde(default)]
    offset: i32,
}

fn default_limit() -> i32 { 20 }

#[derive(Serialize, Deserialize, Debug)]
struct NewMovie {
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tagline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vote_average: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vote_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    popularity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    release_date: Option<String>,
}

// ---------- Config Structs ----------
#[derive(Deserialize)]
struct Config {
    data: ConfigData,
}

#[derive(Deserialize)]
struct ConfigData {
    supabase_api_key: String,
    supabase_url: String,
}

// ---------- Load Config ----------
fn load_config() -> (String, String) {
    // Erst versuchen aus Umgebungsvariablen zu laden (für Railway/Fly.io)
    let api_key = std::env::var("SUPABASE_API_KEY")
        .unwrap_or_else(|_| {
            // Fallback zu config.json für lokale Entwicklung
            let config_content = fs::read_to_string("config.json")
                .expect("Failed to read config.json");
            let config: Config = serde_json::from_str(&config_content)
                .expect("Failed to parse config.json");
            config.data.supabase_api_key
        });
    
    let url = std::env::var("SUPABASE_URL")
        .unwrap_or_else(|_| {
            let config_content = fs::read_to_string("config.json")
                .expect("Failed to read config.json");
            let config: Config = serde_json::from_str(&config_content)
                .expect("Failed to parse config.json");
            config.data.supabase_url
        });
    
    (api_key, url)
}

// ---------- Supabase Client Helper ----------
fn supabase_client() -> (Client, HeaderMap, String) {
    let (api_key, url) = load_config();
    
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&api_key).unwrap());
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
    );
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("Prefer", HeaderValue::from_static("return=representation"));

    (Client::new(), headers, url)
}

// ---------- Main ----------
#[tokio::main]
async fn main() {
    // Port für Railway/Fly.io (dynamisch) oder lokal 8080
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let app = Router::new()
        .route("/", get(serve_frontend))
        .route("/api", get(root_handler))
        .route("/api/movies", get(get_movies).post(create_movie))
        .route("/api/movies/:id", delete(delete_movie))
        .route("/api/test-insert", post(test_insert))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        );

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap();

    println!("🚀 Server running → http://0.0.0.0:{}", port);
    println!("📊 API endpoints:");
    println!("  - GET    /api/movies");
    println!("  - POST   /api/movies");
    println!("  - DELETE /api/movies/:id");

    axum::serve(listener, app)
        .await
        .unwrap();
}

// ---------- Handlers ----------
async fn serve_frontend() -> impl IntoResponse {
    Html(include_str!("../frontend/index.html"))
}

async fn root_handler() -> Json<ApiInfo> {
    Json(ApiInfo {
        message: "Movie API Server".to_string(),
        endpoints: vec![
            "GET /api/movies".to_string(),
            "POST /api/movies".to_string(),
            "DELETE /api/movies/:id".to_string(),
        ],
    })
}

async fn get_movies(Query(params): Query<PaginationParams>) -> Json<Vec<Movie>> {
    let (client, headers, supabase_url) = supabase_client();
    let url = format!(
        "{}/rest/v1/movies?limit={}&offset={}",
        supabase_url,
        params.limit,
        params.offset
    );

    let res = client.get(&url).headers(headers).send().await.unwrap();
    let movies: Vec<Movie> = res.json().await.unwrap();

    Json(movies)
}

async fn create_movie(Json(payload): Json<NewMovie>) -> Result<(StatusCode, Json<Movie>), (StatusCode, String)> {
    let (client, headers, supabase_url) = supabase_client();
    let url = format!("{}/rest/v1/movies", supabase_url);

    let res = client
        .post(&url)
        .headers(headers)
        .json(&payload)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Request error: {:?}", e)))?;

    let movies: Vec<Movie> = res.json().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Parse error: {:?}", e)))?;

    movies.first()
        .cloned()
        .map(|m| (StatusCode::CREATED, Json(m)))
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "No movie returned".to_string()))
}

async fn delete_movie(Path(id): Path<i32>) -> Result<StatusCode, StatusCode> {
    let (client, headers, supabase_url) = supabase_client();
    let url = format!("{}/rest/v1/movies?id=eq.{}", supabase_url, id);

    let res = client.delete(&url).headers(headers).send().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if res.status().is_success() { Ok(StatusCode::NO_CONTENT) } else { Err(StatusCode::NOT_FOUND) }
}

// ---------- Test Endpoint ----------
async fn test_insert() -> Result<String, StatusCode> {
    let (client, headers, supabase_url) = supabase_client();
    let url = format!("{}/rest/v1/movies", supabase_url);

    let test_data = serde_json::json!({ "title": "Test Movie" });

    let res = client.post(&url).headers(headers).json(&test_data).send().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let status = res.status();
    let body = res.text().await.unwrap_or_default();

    Ok(format!("Status: {}\nBody: {}", status, body))
}