use actix_web::{dev::ServiceRequest, dev::ServiceResponse, web, App, HttpServer, HttpResponse, Error, middleware::Logger};
use actix_web::dev::Service;
use actix_web::middleware::Compat;
use futures_util::future::{ok, Either, Ready};

pub fn ip_filter_middleware(
    allowed_ip: &'static str,
) -> Compat<impl Fn(ServiceRequest, &mut actix_web::dev::ServiceFromFn<_>) -> _ + Clone> {
    actix_web::middleware::from_fn(move |req: ServiceRequest, srv| {
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown");

        if ip.starts_with(allowed_ip) {
            Either::Left(srv.call(req))
        } else {
            Either::Right(ok(req.into_response(
                HttpResponse::Unauthorized()
                    .body("Unauthorized IP")
                    .map_into_right_body(),
            )))
        }
    })
}
