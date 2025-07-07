use actix_web::{
    HttpResponse,
    web::{self, ServiceConfig},
};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::resource("/order")
            .route(web::get().to(|| HttpResponse::Ok()))
            .route(web::head().to(|| HttpResponse::Ok())),
    );
}
