use std::io;

use actix_web::{App, HttpServer, middleware::Logger, web};
use confik::{Configuration, EnvSource};
use env_logger::Env;
use fred::prelude::ClientLike;
use sqlx::sqlite::SqlitePoolOptions;

mod inventory;
mod notification;
mod order;
mod payment;
mod product;
mod route;
mod user;

#[derive(Debug, PartialEq, Configuration)]
struct AppConfig {
    valkey_url: String,
    database_url: String,
}

#[derive(Clone)]
struct AppData {
    valkey: fred::prelude::Pool,
    sqlite: sqlx::sqlite::SqlitePool,
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().expect("dotenv");
    env_logger::init_from_env(Env::default_filter_or(Env::default(), "info"));

    let app_config = AppConfig::builder()
        .override_with(EnvSource::new())
        .try_build()
        .expect("app config");

    let sqlite_pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect(&app_config.database_url)
        .await
        .expect("sqlite pool");

    let valkey_config =
        fred::prelude::Config::from_url(&app_config.valkey_url).expect("valkey config");
    let valkey_pool = fred::types::Builder::from_config(valkey_config)
        .build_pool(1)
        .expect("valkey pool");
    valkey_pool.init().await.expect("init pool");

    let app_data = AppData {
        valkey: valkey_pool,
        sqlite: sqlite_pool,
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/v1")
                    .configure(inventory::config)
                    .configure(notification::config)
                    .configure(order::config)
                    .configure(payment::config)
                    .configure(product::config)
                    .configure(route::config)
                    .configure(user::config),
            )
    })
    .workers(1)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
