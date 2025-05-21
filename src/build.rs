use models::{SharedState, UpdateMessage};
use serde::{Deserialize, Serialize};

use crate::models;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Serialize, Deserialize, Debug)]
struct MyCommand {
    command: String,
    title: String,
    icon: Option<String>,
}

fn read_commands() -> Result<Vec<MyCommand>, Box<dyn std::error::Error>> {
    let file_path = "commands.json";
    let json_string = std::fs::read_to_string(file_path)?; // Use BufReader for efficient reading
    let commands: Vec<MyCommand> = serde_json::from_str(&json_string)?;

    Ok(commands)
}

/// This starts the updater in the background and broadcasts its output
pub async fn build(state: Arc<SharedState>) {
    let commads = read_commands();

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
                output: line.clone(),
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
