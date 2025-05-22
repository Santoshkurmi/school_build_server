use actix_web::{web, HttpRequest};

use crate::models::SharedState;

pub async fn is_authorised_client(req: &HttpRequest,state: web::Data<SharedState>) -> bool {

    let allowed_ips = state.config.lock().await.allowed_ips.clone();


    let conn_info = req.connection_info(); // extend lifetime
    let real_ip = conn_info.realip_remote_addr().unwrap_or("unknown");

    allowed_ips.contains(&real_ip.to_string())
}
