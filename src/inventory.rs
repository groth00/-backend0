use actix_web::{HttpResponse, Responder, get, web::ServiceConfig};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(inventory);
}

#[get("/inventory")]
async fn inventory() -> impl Responder {
    HttpResponse::Ok()
}
