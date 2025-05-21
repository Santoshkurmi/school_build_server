
use actix_web::{post, web, HttpResponse, Responder};
use models::{SharedState, UpdateMessage};

use crate::{ models};


/*
|--------------------------------------------------------------------------
| This abort the build process forcefully
|--------------------------------------------------------------------------
|
*/
#[post("/abort")]
pub async fn abort(state: web::Data<SharedState>) -> impl Responder {
    let mut flag = state.is_building.lock().await;
    if *flag {
        let process_state = state.get_ref().clone();
        let mut handle = process_state.builder_handle.lock().await;
        if let Some(handle) = handle.take() {
            handle.abort();
            *flag = false;

            let msg = UpdateMessage {
                step: "0".to_string(),
                status: "aborted".to_string(),
                output: "Done Aborting".to_string(),
            };  
            let json_str = serde_json::to_string(&msg).unwrap();
            return HttpResponse::Ok().body(json_str);

        }
            }//if running
        let msg = UpdateMessage {
                step: "0".to_string(),
                status: "Build is not running already".to_string(),
                output: "It been sleeping".to_string(),
            };  
            let json_str = serde_json::to_string(&msg).unwrap();
            return HttpResponse::Ok().body(json_str);

}