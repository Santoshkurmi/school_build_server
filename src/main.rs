use actix_web::body::MessageBody;
use actix_web::cookie::time::Error;
use actix_web::dev::{Service as _, ServiceRequest, ServiceResponse};
use actix_web::middleware::{self, Next};
use actix_web::{
    App, Either, HttpResponse, HttpServer,
    web::{self},
};
use auth::is_authorised_client;
use build_init::build_initialize;
use futures_util::future::FutureExt;
use handle_abort::abort;
use handle_is_building::is_building;
use handle_socket::connect_and_stream_ws;
use handle_ssl::load_ssl_certificate;
use models::{Config, SharedState};
use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
mod auth;
mod build;
mod build_init;
mod handle_abort;
mod handle_is_building;
mod handle_socket;
mod models;
mod util;
mod handle_error_success;
mod handle_ssl;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let file_path = "config.json";
    let json_string: String = std::fs::read_to_string(file_path)?; // Use BufReader for efficient reading
    let config: Config = serde_json::from_str(&json_string)?;

    let port = config.port;

    let (sender, _) = broadcast::channel(100);
    let state = SharedState {
        buffer: Arc::new(Mutex::new(Vec::new())),
        sender,
        package_name: Arc::new(Mutex::new(None)),
        is_building: Arc::new(Mutex::new(false)),
        builder_handle: Arc::new(Mutex::new(None)),
        token: Arc::new(Mutex::new(None)),
        config: Arc::new(Mutex::new(config)),
    };

    let config = state.config.lock().await.clone();
    let certificate_path = config.certificate_path;
    let certificate_key_path = config.certificate_key_path;

    let builder = load_ssl_certificate(certificate_path,certificate_key_path).await;

    println!("Server listening on port {}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(connect_and_stream_ws) //to stream the update process output //connect
            .service(build_initialize) //to execute the update process //build
            .service(is_building) //to check if the update process is running //is_building
            .service(abort) //to abort the update process  //abort
    })
    // .bind_openssl( format!( "0.0.0.0:{}",port), builder)?
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
