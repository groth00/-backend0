use sqlx::Row;

use actix_web::{
    HttpResponse, Responder, get,
    web::{self, ServiceConfig},
};

use crate::AppData;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(greet);
}

#[get("/ping")]
async fn ping() -> impl Responder {
    "pong"
}

#[get("/greet/{name}")]
async fn greet(name: web::Path<String>, data: web::Data<AppData>) -> impl Responder {
    if let Ok(mut conn) = data.sqlite.acquire().await {
        let name = name.into_inner();
        if let Ok(resp) = sqlx::query(r#"SELECT ?1 as name"#)
            .bind(&name)
            .fetch_one(&mut *conn)
            .await
        {
            HttpResponse::Ok().body(format!("hello {}", resp.get::<&str, usize>(0)))
        } else {
            HttpResponse::InternalServerError().body("query failed")
        }
    } else {
        HttpResponse::InternalServerError().body("failed to acquire connection from pool")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::{App, test, web::Data};
    use fred::prelude::ClientLike;
    use sqlx::sqlite::SqlitePoolOptions;

    #[actix_web::test]
    async fn greet_works() {
        unsafe {
            std::env::set_var("DATABASE_URL", "sqlite::memory:");
            std::env::set_var("VALKEY_URL", "valkey://127.0.0.1:6379");
        }

        let sqlite_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .expect("sqlite pool");

        let valkey_config = fred::prelude::Config::from_url(&std::env::var("VALKEY_URL").unwrap())
            .expect("valkey config");

        let valkey_pool = fred::types::Builder::from_config(valkey_config)
            .build_pool(1)
            .expect("valkey pool");
        valkey_pool.init().await.expect("init pool");

        let app_data = AppData {
            valkey: valkey_pool,
            sqlite: sqlite_pool,
        };

        let app = test::init_service(App::new().app_data(Data::new(app_data)).service(greet)).await;
        let req = test::TestRequest::get().uri("/greet/rakshasa").to_request();
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        assert_eq!(body, "hello rakshasa");
    }

    #[actix_web::test]
    async fn ping_works() {
        let app = test::init_service(App::new().service(ping)).await;
        let req = test::TestRequest::get().uri("/ping").to_request();
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        assert_eq!(body, "pong");
    }
}
