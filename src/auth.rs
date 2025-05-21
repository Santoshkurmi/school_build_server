use actix_web::HttpRequest;

pub fn is_authorised_client(req: &HttpRequest) -> bool {
    let allowed_ips = ["127.0.0.1", "::1", "192.168.1.100"];

    let conn_info = req.connection_info(); // extend lifetime
    let real_ip = conn_info.realip_remote_addr().unwrap_or("unknown");

    allowed_ips.contains(&real_ip)
}
