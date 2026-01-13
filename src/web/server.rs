//! Web server for configuration UI
//!
//! This module provides an Actix-web server with HTTP handlers for
//! configuration management and channel testing.

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_files as fs;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::config::AppConfig;
use crate::channels::r#trait::NotificationChannel;

/// Start the web server on the specified port
pub async fn start_web_server(config_path: PathBuf, port: u16, open_browser: bool) -> std::io::Result<()> {
    let config_data = web::Data::new(Mutex::new(config_path));

    println!("🚀 Starting Claude Code Notifications Web UI...");
    println!("📍 URL: http://localhost:{}", port);
    println!("📁 Config file: {:?}", config_data);

    // Open browser if requested
    if open_browser {
        let url = format!("http://localhost:{}", port);
        if let Err(e) = open::that(&url) {
            eprintln!("⚠️  Warning: Failed to open browser: {}", e);
            eprintln!("   Please open the URL manually in your browser.");
        }
    }

    HttpServer::new(move || {
        App::new()
            .app_data(config_data.clone())
            .service(index)
            .service(api_get_config)
            .service(api_save_config)
            .service(api_test_channel)
            .service(api_list_channels)
            .service(fs::Files::new("/static", "./static").show_files_listing())
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

/// Serve the main index page
#[actix_web::get("/")]
async fn index() -> HttpResponse {
    // Try to read index.html from static directory
    let content_result = tokio::fs::read("./static/index.html").await;

    let content = if content_result.is_err() {
        // Fallback to parent directory
        tokio::fs::read("../static/index.html").await
    } else {
        content_result
    };

    match content {
        Ok(html) => HttpResponse::Ok()
            .content_type(mime::TEXT_HTML_UTF_8.to_string())
            .body(html),
        Err(_) => HttpResponse::NotFound().body("index.html not found"),
    }
}

/// GET /api/config - Get current configuration
#[actix_web::get("/api/config")]
async fn api_get_config(config_path: web::Data<Mutex<PathBuf>>) -> impl Responder {
    let path = config_path.lock().unwrap().clone();
    match load_config_from_path(&path) {
        Ok(config) => HttpResponse::Ok().json(config),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to load config: {}", e)
        })),
    }
}

/// POST /api/config - Save configuration
#[actix_web::post("/api/config")]
async fn api_save_config(
    config_path: web::Data<Mutex<PathBuf>>,
    new_config: web::Json<AppConfig>,
) -> impl Responder {
    let path = config_path.lock().unwrap().clone();
    match save_config_to_path(&new_config, &path) {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "status": "ok",
            "message": "Configuration saved successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to save config: {}", e)
        })),
    }
}

/// POST /api/test/{channel_id} - Test a specific channel
#[actix_web::post("/api/test/{channel_id}")]
async fn api_test_channel(
    path: web::Path<String>,
    config_data: web::Data<Mutex<PathBuf>>,
) -> impl Responder {
    let channel_id = path.into_inner();
    let config_path = config_data.lock().unwrap().clone();

    // Load current configuration
    let app_config = match load_config_from_path(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to load config: {}", e)
            }));
        }
    };

    // Get channel configuration
    let channel_config = match app_config.channels.get(&channel_id) {
        Some(cfg) => cfg.clone(),
        None => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Channel '{}' not found", channel_id)
            }));
        }
    };

    // Get the channel type from config (defaults to channel_id for backward compatibility)
    let channel_type = if channel_config.channel_type.is_empty() {
        channel_id.clone()
    } else {
        channel_config.channel_type.clone()
    };

    // Test the channel based on type
    let result = match channel_type.as_str() {
        "system" => {
            use crate::channels::SystemChannel;
            let channel = SystemChannel::new();
            tokio::task::spawn_blocking(move || {
                tokio::runtime::Runtime::new().unwrap().block_on(channel.test(&channel_config))
            })
            .await
            .unwrap()
        }
        "wechat" => {
            use crate::channels::WeChatChannel;
            let channel = WeChatChannel::new();
            channel.test(&channel_config).await
        }
        "feishu" => {
            use crate::channels::FeishuChannel;
            let channel = FeishuChannel::new();
            channel.test(&channel_config).await
        }
        "dingtalk" => {
            use crate::channels::DingTalkChannel;
            let channel = DingTalkChannel::new();
            channel.test(&channel_config).await
        }
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Unknown channel type: {}", channel_type)
            }));
        }
    };

    match result {
        Ok(msg) => HttpResponse::Ok().json(serde_json::json!({
            "status": "ok",
            "message": msg
        })),
        Err(e) => HttpResponse::Ok().json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}

/// GET /api/channels - List all available channels
#[actix_web::get("/api/channels")]
async fn api_list_channels() -> impl Responder {
    use crate::channels::ChannelRegistry;
    let registry = ChannelRegistry::new();
    let channels = registry.list_channels();

    HttpResponse::Ok().json(serde_json::json!({
        "channels": channels
    }))
}

// Helper functions to avoid module import issues
fn load_config_from_path(path: &PathBuf) -> Result<AppConfig, String> {
    crate::config::load_config_from_path(path)
        .map_err(|e| e.to_string())
}

fn save_config_to_path(config: &AppConfig, path: &PathBuf) -> Result<(), String> {
    crate::config::save_config_to_path(config, path)
        .map_err(|e| e.to_string())
}
