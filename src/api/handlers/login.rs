use crate::models::UserData;
use actix_web::{post, web, HttpResponse, Responder};
use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::Utc;
use std::env;

use crate::services::user_service;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub exp: u64,
    pub role: String,
}

#[post("/login")]
pub async fn login(
    pool: web::Data<crate::AppState>,
    user_data: web::Json<UserData>,
) -> impl Responder {
    let user = match user_service::verify_login(&pool.db, &user_data).await {
        Ok(u) => u,
        Err(e) => return e,
    };
    
    let claims = Claims {
        sub: user.id,
        exp: (Utc::now() + chrono::Duration::hours(2)).timestamp() as u64,
        role: user.role.clone(),
    };

    let secret_key = env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set in .env file");
    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(secret_key.as_ref())) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Login exitoso",
        "token": token,
        "role": user.role,
    }))
}
