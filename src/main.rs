use build_init::build_initialize;
use handle_abort::abort;
use handle_socket::connect_and_stream_ws;
use handle_is_building::is_building;
use models::{SharedState};
use std::{sync::Arc};
use tokio::{
    sync::{broadcast, Mutex},
};
use actix_web::{ web::{self}, App, HttpServer};


mod models;
mod  handle_socket;
mod  build;
mod  build_init;
mod  handle_is_building;
mod handle_abort;
mod util;




#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let (sender, _) = broadcast::channel(100);
    let state = SharedState {
        buffer: Arc::new(Mutex::new(Vec::new())),
        sender,
        is_building: Arc::new(Mutex::new(false)),
        builder_handle: Arc::new(Mutex::new(None)),
        token: Arc::new(Mutex::new(None)),
    };


    HttpServer::new(move || {
        App::new()

            .app_data(web::Data::new(state.clone()))
            .service(build_initialize)   //to execute the update process //build
            .service(is_building) //to check if the update process is running //is_building
            .service(connect_and_stream_ws) //to stream the update process output //connect
            .service(abort) //to abort the update process  //abort
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}