use actix_web::{HttpResponse, Responder, get, web::ServiceConfig};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(notify);
}

#[get("/notify")]
async fn notify() -> impl Responder {
    HttpResponse::Ok()
}
