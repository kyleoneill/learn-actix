use actix_web::{body::BoxBody, http::header::ContentType, get, post, put, web::{self, Json}, HttpResponse, Responder, HttpRequest, dev::HttpServiceFactory};
use serde::{Serialize, Deserialize};
use sqlx::sqlite::SqliteQueryResult;

#[path = "../auth.rs"] mod auth;

use crate::Pool;
use crate::errors;
use errors::CustomError;

use chrono::{DateTime, Utc};

#[derive(Serialize)]
pub struct Achievement {
    id: i64,
    name: String,
    image: String
}

#[derive(Deserialize)]
pub struct WebAchievement {
    name: String,
    image: String
}

impl Responder for Achievement {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

#[get("")]
pub async fn show_achievements(pool: web::Data<Pool>) -> Result<Json<Vec<Achievement>>, CustomError> {
    match get_all_achievements(pool).await {
        Ok(result) => Ok(web::Json(result)),
        Err(_) => Err(CustomError {error_type: errors::ErrorType::InternalError, message: Some(format!("Failed to get achievement data"))})
    }
}

#[get("{id}")]
pub async fn get_individual_achievement(req: HttpRequest) -> Result<Achievement, CustomError> {
    match req.match_info().get("id").unwrap().parse::<u32>() {
        Ok(valid_id) => match valid_id {
            1 => Ok(Achievement { name: "You're cool!".to_owned(), image: "AASdkljfasdjf".to_owned(), id: 1 }),
            _ => Err(CustomError {error_type: errors::ErrorType::NotFound, message: Some(format!("Could not find achievement with id {}", valid_id))})
        }
        Err(_) => Err(CustomError {error_type: errors::ErrorType::BadClientData, message: Some(format!("The provided id must be numeric"))})
    }
}

#[post("")]
pub async fn post_achievement(achievement: web::Json<WebAchievement>) -> impl Responder {
    println!("Got achievement with name: {}", achievement.name);
    HttpResponse::Created()
}

#[put("/unlock/{id}")]
pub async fn unlock_achievement(req: HttpRequest, pool: web::Data<Pool>) -> Result<impl Responder, CustomError> {
    let user = auth::authenticate_request(&req, &pool, auth::AuthType::user).await?; // Not the right way to do this
    let achievement_id = req.match_info().get("id").unwrap().parse::<i64>()
        .map_err(|_| CustomError {error_type: errors::ErrorType::BadClientData, message: Some(format!("Invalid achievement id"))})?;
    let achievement = get_achievement_by_id(&pool, achievement_id).await
        .map_err(|_| CustomError {error_type: errors::ErrorType::NotFound, message: Some(format!("That achievement does not exist"))})?;
    match unlock_achievement_sql(&pool, user.id.unwrap(), achievement.id).await {
        Ok(_) => Ok(HttpResponse::Ok()),
        Err(_) => Err(CustomError {error_type: errors::ErrorType::InternalError, message: None})
    }
}

pub fn controller() -> impl HttpServiceFactory {
    web::scope("/achievements")
        .service(show_achievements)
        .service(post_achievement)
        .service(get_individual_achievement)
        .service(unlock_achievement)
}

async fn unlock_achievement_sql(pool: &web::Data<Pool>, user_id: i64, achievement_id: i64) -> Result<SqliteQueryResult, sqlx::Error> {
    let dt: DateTime<Utc> = Utc::now();
    let time = dt.timestamp();
    sqlx::query!(
        r#"
        REPLACE INTO userAchievements (user_id, achievement_id, unlocked, time_unlocked) VALUES ($1, $2, $3, $4)
        "#,
        user_id,
        achievement_id,
        1 as i64,
        time
    ).execute(pool.as_ref()).await
}

async fn get_all_achievements(pool: web::Data<Pool>) -> Result<Vec<Achievement>, sqlx::Error> {
    sqlx::query_as!(
        Achievement,
        r#"
        SELECT * FROM achievements LIMIT 50
        "#
    ).fetch_all(pool.as_ref()).await
}

async fn get_achievement_by_id(pool: &web::Data<Pool>, id: i64) -> Result<Achievement, sqlx::Error> {
    sqlx::query_as!(
        Achievement,
        r#"
        SELECT * FROM achievements WHERE id = $1
        "#,
        id
    ).fetch_one(pool.as_ref()).await
}