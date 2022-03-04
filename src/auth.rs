use actix_web::{HttpRequest, web};
use sqlx::Executor;
use crate::Pool;

#[path = "endpoints/users.rs"] mod users;
use users::User;

use crate::errors;
use errors::CustomError;

pub enum AuthType {
    user,
    admin
}

pub async fn authenticate_request(req: &HttpRequest, pool: &web::Data<Pool>, auth_type: AuthType) -> Result<User, CustomError> {
    match req.headers().get(actix_web::http::header::AUTHORIZATION) {
        Some(header) => {
            let token = header.to_str().unwrap();
            let record = sqlx::query!(r#"SELECT username FROM tokens WHERE token = $1"#, token).fetch_one(pool.as_ref()).await
                .map_err(|_| CustomError {error_type: errors::ErrorType::BadClientData, message: Some(format!("invalid token in authorization header"))})?;
            let user = sqlx::query_as!(User, r#"SELECT * FROM users WHERE username = $1"#, record.username).fetch_one(pool.as_ref()).await
                .map_err(|_| CustomError {error_type: errors::ErrorType::InternalError, message: None})?;
            match auth_type {
                AuthType::user => Ok(user),
                AuthType::admin => {
                    match user.is_admin {
                        true => Ok(user),
                        false => Err(CustomError {error_type: errors::ErrorType::InvalidAuth, message: Some(format!("admin authorization required"))})
                    }
                }
            }
        },
        None => Err(CustomError {error_type: errors::ErrorType::BadClientData, message: Some(format!("missing authorization header"))})
    }
}
