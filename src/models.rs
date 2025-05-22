use serde::{Deserialize, Serialize};
use std::{sync::Arc};
use tokio::{
    sync::{broadcast, Mutex},
};
use actix_web::rt::task::JoinHandle;


/*
|--------------------------------------------------------------------------
| This is the payload that will be sent to the client to inform about the build state
|--------------------------------------------------------------------------
|
*/

#[derive(Serialize)]
pub struct UpdateMessage {
    pub step: String,
    pub  status: String,
    pub output: String,
}

/*
|--------------------------------------------------------------------------
| This is the paylaod to send for build state like token and status only
|--------------------------------------------------------------------------
|
*/

#[derive(Serialize)]
pub struct BuildState {
    pub token: Option<String>,
    pub is_running: bool,
}

#[derive(Deserialize)]
pub struct ConnectParams {
    pub token: String,
}
/*
|--------------------------------------------------------------------------
| This is the type and payload for storing the build output in buffer, and its state like whether its updating or not
| as well as its spawn process to kill the process later
|--------------------------------------------------------------------------
|
*/

type SharedBuffer = Arc<Mutex<Vec<UpdateMessage>>>;
type SharedSender = broadcast::Sender<String>;

#[derive(Clone)]
pub struct SharedState {
    pub buffer: SharedBuffer,
    pub sender: SharedSender,
    pub is_building: Arc<Mutex<bool>>,
    pub token: Arc<Mutex<Option<String>>>, // this store random 32 character string to identify the build process from client browser to connect to websocket
    pub builder_handle: Arc<Mutex<Option<JoinHandle<()>>>>, //it store the handle of the build process later to kill it
    pub config: Arc<Mutex<Config>>,
}


#[derive(Serialize, Deserialize)]
pub struct MyCommand {
    command: String,
    title: String,
}


/*
|--------------------------------------------------------------------------
| This is the config payload of the server to make it indepenent for loose coupling
|--------------------------------------------------------------------------
|
*/

#[derive(Serialize,Deserialize)]
pub struct Config {
    pub name: String,
    pub allowed_ips: Vec<String>,
    pub on_success: String,
    pub on_failure: String,
    pub port: u16,
    pub commands: Vec<MyCommand>,
}