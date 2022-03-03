use actix_web::{HttpRequest, web};
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
            match sqlx::query!(
                r#"
                SELECT username FROM tokens WHERE token = $1
                "#,
                token,
            ).fetch_one(pool.as_ref()).await {
                Ok(record) => {
                    match sqlx::query_as!(
                        User,
                        r#"
                        SELECT * FROM users WHERE username = $1
                        "#,
                        record.username
                    ).fetch_one(pool.as_ref()).await {
                        Ok(user_record) => Ok(user_record),
                        Err(_e) => Err(CustomError {error_type: errors::ErrorType::InternalError, message: None})
                    }
                }
                Err(_) => Err(CustomError {error_type: errors::ErrorType::BadClientData, message: Some(format!("invalid token in authorization header"))})
            }
        },
        None => Err(CustomError {error_type: errors::ErrorType::BadClientData, message: Some(format!("missing authorization header"))})
    }
}
