use actix_web::{
    HttpResponse, Responder, get,
    web::{self, ServiceConfig},
};

use crate::AppData;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(greet);
}

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>, data: web::Data<AppData>) -> impl Responder {
    if let Ok(mut conn) = data.sqlite.acquire().await {
        if let Ok(resp) = sqlx::query!("SELECT current_time")
            .fetch_one(&mut *conn)
            .await
        {
            HttpResponse::Ok().body(format!(
                "hello {}, the current time is {} (UTC)",
                name,
                resp.current_time.unwrap()
            ))
        } else {
            HttpResponse::InternalServerError().body("query failed")
        }
    } else {
        HttpResponse::InternalServerError().body("failed to acquire connection from pool")
    }
}
