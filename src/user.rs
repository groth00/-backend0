use std::fmt::{self, Display, Formatter};

use actix_web::{
    HttpResponse, ResponseError, Result, delete, get,
    http::StatusCode,
    post, put,
    web::{self, Data, ServiceConfig},
};
use serde::{Deserialize, Serialize};

use crate::AppData;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .service(get_user)
            .service(create_user)
            .service(delete_user)
            .service(update_user),
    );
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppError {
    ConnectionError(String),
    QueryError(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionError(s) => write!(f, "{}", s),
            Self::QueryError(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse<'a> {
    code: u16,
    message: &'a str,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        let (status_code, msg) = match self {
            AppError::ConnectionError(s) => (StatusCode::SERVICE_UNAVAILABLE, s),
            AppError::QueryError(s) => (StatusCode::INTERNAL_SERVER_ERROR, s),
        };

        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: msg,
        };

        HttpResponse::build(status_code).json(error_response)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct User {
    name: String,
    username: String,
    email: String,
}

#[get("/{username}")]
async fn get_user(username: web::Path<String>, data: Data<AppData>) -> Result<HttpResponse> {
    let username = username.into_inner();

    let resp = sqlx::query!(
        "SELECT name, username, email FROM users WHERE username = ?1",
        (username)
    )
    .fetch_one(&data.sqlite)
    .await
    .map_err(|e| AppError::QueryError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(User {
        name: resp.name,
        username: resp.username,
        email: resp.email,
    }))
}

#[post("/")]
async fn create_user(input: web::Json<User>, data: Data<AppData>) -> Result<HttpResponse> {
    let mut conn = data
        .sqlite
        .acquire()
        .await
        .map_err(|e| AppError::ConnectionError(e.to_string()))?;

    let _resp = sqlx::query!(
        "INSERT INTO users(name, username, email) VALUES(?1, ?2, ?3)",
        input.name,
        input.username,
        input.email
    )
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::QueryError(e.to_string()))?;

    Ok(HttpResponse::Created().finish())
}

#[delete("/{username}")]
async fn delete_user(username: web::Path<String>, data: Data<AppData>) -> Result<HttpResponse> {
    let username = username.into_inner();

    let mut conn = data
        .sqlite
        .acquire()
        .await
        .map_err(|e| AppError::ConnectionError(e.to_string()))?;

    let _resp = sqlx::query!("DELETE FROM users WHERE username = ?1", (username))
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::QueryError(e.to_string()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[put("/")]
async fn update_user(user: web::Json<User>, data: Data<AppData>) -> Result<HttpResponse> {
    let mut conn = data
        .sqlite
        .acquire()
        .await
        .map_err(|e| AppError::ConnectionError(e.to_string()))?;

    let _resp = sqlx::query!(
        "UPDATE users SET name = ?1, email = ?2 WHERE username = ?3",
        user.name,
        user.email,
        user.username
    )
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::QueryError(e.to_string()))?;

    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::{
        App,
        test::{TestRequest, call_and_read_body_json, call_service, init_service},
    };
    use fred::prelude::ClientLike;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup() -> AppData {
        unsafe {
            std::env::set_var("DATABASE_URL", "sqlite::memory:");
            std::env::set_var("VALKEY_URL", "valkey://127.0.0.1:6379");
        }

        let sqlite_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .expect("sqlite pool");

        sqlx::migrate!().run(&sqlite_pool).await.expect("migrate");
        sqlx::query_file!("migrations/test/insert_test_data.sql")
            .execute(&sqlite_pool)
            .await
            .expect("migrate insert");

        let valkey_config = fred::prelude::Config::from_url(&std::env::var("VALKEY_URL").unwrap())
            .expect("valkey config");

        let valkey_pool = fred::types::Builder::from_config(valkey_config)
            .build_pool(1)
            .expect("valkey pool");
        valkey_pool.init().await.expect("init pool");

        AppData {
            valkey: valkey_pool,
            sqlite: sqlite_pool,
        }
    }

    #[actix_web::test]
    async fn user_create() {
        let app_data = setup().await;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(app_data))
                .service(create_user),
        )
        .await;

        let user = User {
            name: "unused".into(),
            username: "unusedusername".into(),
            email: "unused@example.com".into(),
        };

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(user)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn user_get() {
        let app_data = setup().await;

        let app = init_service(App::new().app_data(Data::new(app_data)).service(get_user)).await;
        let req = TestRequest::get().uri("/javascript").to_request();
        let json: User = call_and_read_body_json(&app, req).await;

        assert_eq!(
            json,
            User {
                name: "js".into(),
                username: "javascript".into(),
                email: "javascript@example.com".into(),
            }
        );
    }

    #[actix_web::test]
    async fn user_delete() {
        let app_data = setup().await;

        let app = init_service(
            App::new()
                .app_data(Data::new(app_data))
                .service(delete_user),
        )
        .await;
        let req = TestRequest::delete().uri("/foobar").to_request();
        let resp = call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }

    #[actix_web::test]
    async fn user_update() {
        let app_data = setup().await;

        let updated_user = User {
            name: "ts".into(),
            username: "typescript".into(),
            email: "typescript@example.com".into(),
        };

        let app = init_service(
            App::new()
                .app_data(Data::new(app_data))
                .service(update_user),
        )
        .await;
        let req = TestRequest::put()
            .uri("/")
            .set_json(updated_user)
            .to_request();
        let resp = call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
