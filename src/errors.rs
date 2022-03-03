use core::fmt;
use std::fmt::Debug;

use actix_web::{error, HttpResponse, http::{StatusCode, header::ContentType}};

#[allow(dead_code)]
#[derive(Debug)]
pub enum ErrorType {
    InternalError,
    BadClientData,
    NotFound,
    AlreadyExists,
    InvalidAuth
}

#[derive(Debug)]
pub struct CustomError {
    pub error_type: ErrorType,
    pub message: Option<String>
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(message) => write!(f, "{}", message),
            None => match self.error_type {
                ErrorType::InternalError => write!(f, "internal error"),
                ErrorType::BadClientData => write!(f, "bad request"),
                ErrorType::NotFound => write!(f, "resource not found"),
                ErrorType::AlreadyExists => write!(f, "resource already exists"),
                ErrorType::InvalidAuth => write!(f, "invalid authentication")
            }
        }
    }
}

impl error::ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match self.error_type {
            ErrorType::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorType::BadClientData => StatusCode::BAD_REQUEST,
            ErrorType::NotFound => StatusCode::NOT_FOUND,
            ErrorType::AlreadyExists => StatusCode::CONFLICT,
            ErrorType::InvalidAuth => StatusCode::FORBIDDEN
        }
    }
}
