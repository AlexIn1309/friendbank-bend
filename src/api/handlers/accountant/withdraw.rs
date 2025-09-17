// src/api/handlers/accountant/withdraw.rs

use crate::AppState;
use crate::middleware::jwt_auth::Claims;
use actix_web::{post, web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::json;
use std::env;

use crate::models::AccountantData;
use crate::services::accountant::withdraw as accountant_withdraw;

#[post("/withdraw")]
pub async fn withdraw(
    pool: web::Data<AppState>,
    data: web::Json<AccountantData>,
    bearer: BearerAuth,
) -> impl Responder {
    let claims = match decode::<Claims>(
        bearer.token(),
        &DecodingKey::from_secret(env::var("JWT_SECRET").expect("JWT_SECRET must be set").as_ref()),
        &Validation::new(jsonwebtoken::Algorithm::default()),
    ) {
        Ok(token_data) => token_data.claims,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    if claims.role != "accountant" {
        return HttpResponse::Forbidden().json("Acceso denegado. Solo el contador puede realizar esta acción.");
    }

    if data.amount <= 0.0 {
        return HttpResponse::BadRequest().json("La cantidad debe ser mayor a 0.");
    }

    match accountant_withdraw::process_withdrawal(&pool.db, claims.sub, &data).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Retiro realizado con éxito"
        })),
        Err(e) => e,
    }
}
