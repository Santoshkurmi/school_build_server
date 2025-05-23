use std::sync::Arc;
use std::time::Duration;

use rand::{distributions::Alphanumeric, Rng};
use reqwest::Client;
use crate::models::{self, UpdateMessage};
use tokio::io::{AsyncBufReadExt, BufReader};
use crate::models::SharedState;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;

pub fn generate_token(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}


pub async fn read_output_lines(
    stream: Option<impl tokio::io::AsyncRead + Unpin>,
    step: usize,
    status: &str,
    state: &Arc<SharedState>,
) {
    if let Some(output) = stream {
        let reader = BufReader::new(output);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            send_output(state, step, status, &line).await;
        }
    }
}

pub async fn send_output(state: &Arc<SharedState>, step: usize, status: &str, output: &str) {
    let msg = UpdateMessage {
        step: step.to_string(),
        status: status.to_string(),
        output: output.to_string(),
    };
    let json_str = serde_json::to_string(&msg).unwrap();

    let _ = state.sender.send(json_str.clone());

    let mut buf = state.buffer.lock().await;
    buf.push(msg);
}


pub async  fn save_log(log_path:String,logs:String,token:String){

   
    // let home_dir = dirs::home_dir().expect("Home directory not found");

    // let full_path = format!("{}/{}",home_dir.to_string_lossy(),log_path);

    let full_path = log_path;

    // Create logs directory if it doesn't exist
    fs::create_dir_all( &full_path).expect("Failed to create logs directory");

    // Create a file inside ~/logs
    let mut file_path = PathBuf::from(&full_path);

    let now = Local::now();
    file_path.push(format!("{}_{}.log", now.format("%Y-%m-%d_%H-%M-%S"), token ));

    println!("File path: {}", file_path.to_str().unwrap());

    let mut file = File::create(file_path).expect("Failed to create file");
    // writeln!(file, "This is a new log entry!").expect("Failed to write to file");

    file.write_all(logs.as_bytes()).expect("Failed to write to file");

   
}


pub async fn send_to_other_server(path:String,data:String) ->bool{
    let client = Client::new();
    println!("{}",path);
    let res = client
        .post(path)
        .body(data)
        .header("Content-Type", "application/json")
        .timeout(Duration::new(5, 0))
        .send()
        .await;
    match res {
        Ok(response) => {
            let status = response.status();
            if  !status.is_success(){
                println!("failed to send data to other server: {}", status);
                return  false;
            }
            let body = response.text().await.unwrap_or_default();
            println!("Successfully sent data to other server: {}", status);
            println!("Response body: {}", body);
            return  true;
        }
        Err(err) => {
            println!("failed to send data to other server: {}", err);
            return  false;
        } 
    }


}