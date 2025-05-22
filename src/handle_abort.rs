use actix_web::{HttpRequest, HttpResponse, Responder, post, web};
use models::{SharedState, UpdateMessage};

use crate::{auth::is_authorised_client, models};

/*
|--------------------------------------------------------------------------
| This abort the build process forcefully
|--------------------------------------------------------------------------
|
*/
#[post("/abort")]
pub async fn abort(req: HttpRequest, state: web::Data<SharedState>) -> impl Responder {
    if !is_authorised_client(&req,state.clone()).await {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

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
    } //if running
    let msg = UpdateMessage {
        step: "0".to_string(),
        status: "Build is not running already".to_string(),
        output: "It been sleeping".to_string(),
    };
    let json_str = serde_json::to_string(&msg).unwrap();
    return HttpResponse::Ok().body(json_str);
}

