use crate::{build, models::{self, BuildState}, util::generate_token};
use actix_web::{HttpResponse, Responder, post, rt::task::JoinHandle, web};
use build::build;
use models::{SharedState, UpdateMessage};
use std::sync::Arc;

/*
|--------------------------------------------------------------------------
| This just start the build process
|--------------------------------------------------------------------------
|
*/

#[post("/build")]
pub async fn build_initialize(
    _package_name: String,
    state: web::Data<SharedState>,
) -> impl Responder {
    let process_state = state.get_ref().clone();

    let mut flag = state.is_building.lock().await;
    if *flag {
        
        let  token = state.token.lock().await;

        let payload = BuildState {
            token: Some( "/connect?token=".to_string()+ &token.clone().unwrap()),
            is_running:true
        };

        let json_str = serde_json::to_string(&payload).unwrap();

        return HttpResponse::Ok().body(json_str);
    }
    *flag = true; // set as updating

    let process_state_clone = Arc::clone(&state);
    let handle_curent: JoinHandle<()> = actix_web::rt::spawn(async move {
        build(process_state_clone).await;
    });
    let mut handle = process_state.builder_handle.lock().await;
    *handle = Some(handle_curent);

    let mut token = state.token.lock().await;

    let new_token = generate_token(32);
    *token = Some(new_token.clone());

    let payload = BuildState {
        token: Some("/connect?token=".to_string()+ &new_token.clone()),
        is_running:true
    };

    let json_str = serde_json::to_string(&payload).unwrap();


    HttpResponse::Ok().body(json_str)

}
