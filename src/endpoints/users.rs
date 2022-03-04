use actix_web::{http::header::ContentType, post, web, HttpResponse, Responder, dev::HttpServiceFactory};
use serde::{Serialize, Deserialize};
use sqlx::sqlite::SqliteQueryResult;
use crate::Pool;
extern crate bcrypt;

use crate::util;
use util::random_string;

use crate::errors;
use errors::CustomError;

#[derive(Serialize)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub hashed_password: String,
    pub is_admin: bool
}

pub struct Token {
    token: String,
    username: String
}

#[derive(Deserialize)]
pub struct WebUser {
    username: String,
    password: String
}

#[post("")]
pub async fn post_user(user: web::Json<WebUser>, pool: web::Data<Pool>) -> Result<impl Responder, CustomError> {
    match user_exists(&pool, &user.username).await {
        true => Err(CustomError {error_type: errors::ErrorType::AlreadyExists, message: Some(format!("That username is already in use"))}),
        false => {
            match create_user(&pool, user).await {
                Ok(_) => Ok(HttpResponse::Created()),
                Err(_) => Err(CustomError {error_type: errors::ErrorType::InternalError, message: None})
            }
        }
    }
}

#[post("/auth")]
pub async fn auth_user(pool: web::Data<Pool>, user: web::Json<WebUser>) -> Result<impl Responder, CustomError> {
    match verify_user(&pool, &user).await {
        Ok(valid_credentials) => match valid_credentials {
            true => {
                match generate_token(&pool, &user.username).await {
                    Ok(token) => Ok(HttpResponse::Created().content_type(ContentType::json()).body(format!(r#"{{"token":"{}"}}"#, &token))),
                    Err(_) => Err(CustomError {error_type: errors::ErrorType::InternalError, message: None})
                }
            },
            false => Err(CustomError {error_type: errors::ErrorType::BadClientData, message: Some("Invalid credentials".to_owned())})
        },
        Err(_) => Err(CustomError {error_type: errors::ErrorType::BadClientData, message: Some("User does not exist".to_owned())})
    }
}

pub fn controller() -> impl HttpServiceFactory {
    web::scope("/users")
        .service(post_user)
        .service(auth_user)
}

async fn user_exists(pool: &web::Data<Pool>, username: &str) -> bool {
    let foo = sqlx::query_as!(
        User,
        r#"
        SELECT * FROM users WHERE username = $1
        "#,
        username
    ).fetch_one(pool.as_ref()).await;
    match foo {
        Ok(_) => true,
        Err(_) => false
    }
}

async fn verify_user(pool: &web::Data<Pool>, web_user: &web::Json<WebUser>) -> Result<bool, sqlx::Error> {
    match sqlx::query_as!(
        User,
        r#"
        SELECT * FROM users WHERE username = $1
        "#,
        web_user.username
    ).fetch_one(pool.as_ref()).await {
        Ok(user) => Ok(bcrypt::verify(&web_user.password, &user.hashed_password).unwrap()),
        Err(e) => Err(e)
    }
}

async fn generate_token(pool: &web::Data<Pool>, username: &str) -> Result<String, sqlx::Error> {
    let token = random_string(25);
    match sqlx::query!(
        r#"
        REPLACE INTO tokens (token, username) VALUES ($1, $2)
        "#,
        token,
        username
    ).execute(pool.as_ref()).await {
        Ok(_) => Ok(token),
        Err(e) => Err(e)
    }
}

async fn create_user(pool: &web::Data<Pool>, user: web::Json<WebUser>) -> Result<SqliteQueryResult, sqlx::Error> {
    let hashed_password = bcrypt::hash(&user.password, bcrypt::DEFAULT_COST).expect("Failed to hash password");
    sqlx::query!(
        r#"
        INSERT INTO users (username, hashed_password) VALUES ($1, $2)
        "#,
        user.username,
        hashed_password
    ).execute(pool.as_ref()).await
}