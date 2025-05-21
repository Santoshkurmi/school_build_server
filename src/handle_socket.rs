use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_ws::handle;

use crate::models::{ConnectParams, SharedState};




/*
|--------------------------------------------------------------------------
| This is the route to connect to the build process output using websocket
|-----------------------------------------------------------------------
|
*/
#[get("/connect")]
pub async fn connect_and_stream_ws(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<SharedState>,
    query: web::Query<ConnectParams>,
) -> Result<HttpResponse, Error> {


    /*
    |--------------------------------------------------------------------------
    | Handle to check if token is matched or not, this is used in single place, so no need to crate middleware for that
    |--------------------------------------------------------------------------
    |
    */

    let token = &query.token;

    let state = data.as_ref();
    let current_token_lock = state.token.lock().await;
    if current_token_lock.as_deref() != Some(token.as_str()) {
        return Ok(HttpResponse::Unauthorized().body("Invalid token"));
    }


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
