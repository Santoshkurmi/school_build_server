use serde::Serialize;
// use actix_web::{get, post,rt, web, App, HttpResponse,HttpRequest, Error, HttpServer, Responder};
// use actix_ws::{handle, AggregatedMessage};
use tokio::process::Command;
use std::{process::Stdio, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::{broadcast, Mutex},
};
// use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;

use actix_web::{rt, web, App, Error, get,post,HttpRequest, HttpResponse,Responder, HttpServer};
use actix_ws::AggregatedMessage;
use actix_ws::handle;


#[derive(Serialize)]
struct UpdateMessage {
    step: String,
    status: String,
    output: String,
}


type SharedBuffer = Arc<Mutex<Vec<String>>>;
type SharedSender = broadcast::Sender<String>;

#[derive(Clone)]
struct SharedState {
    buffer: SharedBuffer,
    sender: SharedSender,
    pub is_updating: Arc<Mutex<bool>>,
}

#[get("/update")]
async fn update_and_stream_ws(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<SharedState>,
) -> Result<HttpResponse, Error> {
    let (res, mut session, _msg_stream) = handle(&req, stream)?;

    // Send old buffered messages first
    {
        let buf = data.buffer.lock().await;
        for line in buf.iter() {
            let _ = session.text(line.clone()).await;
        }
    }

    // Subscribe to broadcast channel
    let mut rx = data.sender.subscribe();

    // Stream new output to client
    actix_web::rt::spawn(async move {
        while let Ok(line) = rx.recv().await {

          
            if session.text(line).await.is_err() {
                break; // disconnected
            }
        }
    });

    Ok(res)
}

/// This starts the updater in the background and broadcasts its output
async fn run_update_process(state: SharedState) {
    let mut child = Command::new("bash")
        .arg("-c")
        .arg("echo Starting update... && sleep 1 && echo Installing... && sleep 1 && echo Done!")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start");

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let mut step = 1;
        while let Ok(Some(line)) = lines.next_line().await {
            // Broadcast line to all connected clients
            let msg = UpdateMessage {
                step: step.to_string(),
                status: "running".to_string(),
                output: line.clone()
            };
            step += 1;
            let json_str = serde_json::to_string(&msg).unwrap();
            

            let _ = state.sender.send(json_str.clone());

            // Also store in buffer
            let mut buf = state.buffer.lock().await;
            buf.push(json_str.clone());
        }
    }

    let _ = child.wait().await;
}




async fn update_and_stream(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, msg_stream) = actix_ws::handle(&req, stream)?;

    // let mut _msg_stream = msg_stream
    //     .aggregate_continuations()
    //     .max_continuation_size(2_usize.pow(20));

    // Spawn background task
    rt::spawn(async move {
        // You can replace this with your actual updater script/command
        let mut child = Command::new("bash")
            .arg("-c")
            .arg("echo Starting update... && sleep 1 && echo Installing packages... && sleep 1 && echo Done!")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to start update process");

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if session.text(line).await.is_err() {
                    break; // Stop if client disconnects
                }
            }
        }

        let _ = child.wait().await;
    });

    Ok(res)
}

#[get("/is_building")]
async fn is_building() -> impl Responder {
    HttpResponse::Ok().body("yes!")
}

#[post("/update_start")]
async fn update(_package_name: String,state: web::Data<SharedState>) -> impl Responder {
    let process_state = state.get_ref().clone();

    let mut flag = state.is_updating.lock().await;
        if *flag {
            let msg = UpdateMessage {
                step: "0".to_string(),
                status: "running".to_string(),
                output: "Started Already".to_string(),
            };
            let json_str = serde_json::to_string(&msg).unwrap();
            return HttpResponse::Ok().body(json_str);
        }
        *flag = true; // set as updating

        actix_web::rt::spawn(async move {
            run_update_process(process_state).await;
        });

    let msg = UpdateMessage {
        step: "0".to_string(),
        status: "begin running".to_string(),
        output: "Starting update...".to_string(),
    };
    let json_str = serde_json::to_string(&msg).unwrap();

    HttpResponse::Ok().body(json_str)
}

#[post("/stop")]
async fn stop() -> impl Responder {
    HttpResponse::Ok().body("stopped")
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let (sender, _) = broadcast::channel(100);
    let state = SharedState {
        buffer: Arc::new(Mutex::new(Vec::new())),
        sender,
        is_updating: Arc::new(Mutex::new(false)),
    };

    // Start update process once (can also trigger on HTTP endpoint)
    


    HttpServer::new(move || {
        App::new()
            .service(update)
            .app_data(web::Data::new(state.clone()))
            .service(update_and_stream_ws)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}