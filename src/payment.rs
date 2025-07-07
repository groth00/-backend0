use actix_web::{HttpResponse, Responder, get, web::ServiceConfig};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(payment);
}

#[get("/payment")]
async fn payment() -> impl Responder {
    HttpResponse::Ok()
}
