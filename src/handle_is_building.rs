use models::{SharedState, UpdateMessage};

use actix_web::{HttpRequest, HttpResponse, Responder, get, web};

use crate::{
    auth::is_authorised_client,
    models::{self, BuildState},
};

/*
|--------------------------------------------------------------------------
| This provide whether the build process is running or not
|--------------------------------------------------------------------------
|
*/
#[get("/is_building")]
pub async fn is_building(req: HttpRequest, state: web::Data<SharedState>) -> impl Responder {
    if !is_authorised_client(&req,state.clone()).await {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    let flag = state.is_building.lock().await;

    if *flag {
        let token = state.token.lock().await;

        let payload = BuildState {
            token: Some("/connect?token=".to_string() + &token.clone().unwrap()),
            is_running: true,
        };

        let json_str = serde_json::to_string(&payload).unwrap();

        return HttpResponse::Ok().body(json_str);
    } else {
        let payload = BuildState {
            token: None,
            is_running: false,
        };

        let json_str = serde_json::to_string(&payload).unwrap();

        return HttpResponse::Ok().body(json_str);
    }
}
