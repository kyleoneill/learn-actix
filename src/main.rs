extern crate dotenv;
use dotenv::dotenv;
use std::env;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, error, http::header::ContentType, middleware};
#[path = "endpoints/achievements.rs"] mod achievements;
#[path = "endpoints/users.rs"] mod users;

async fn health() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(format!(r#"{{"status":"Online"}}"#))
}

pub type Databse = sqlx::Sqlite;
pub type Pool = sqlx::Pool<Databse>;

mod errors;
mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let pool = Pool::connect(&env::var("DATABASE_URL").unwrap()).await.unwrap();
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Compress::default())
            .service(
                web::scope("/api")
                    .service(achievements::controller())
                    .service(users::controller())
                    .route("/health", web::get().to(health))
            )
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                error::InternalError::from_response(
                    "",
                    HttpResponse::BadRequest()
                        .content_type("application/json")
                        .body(format!(r#"{{"error":"{}"}}"#, err)) // This is not escaped and can create an invalid document
                )
                .into()
            }))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
