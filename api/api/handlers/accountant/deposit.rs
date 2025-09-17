use crate::AppState;
use crate::middleware::jwt_auth::Claims;
use actix_web::{post, web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::json;
use std::env;

use crate::models::AccountantData;
use crate::services::accountant::deposit as accountant_deposit;

#[post("/deposit")]
pub async fn deposit(
    pool: web::Data<AppState>,
    data: web::Json<AccountantData>,
    bearer: BearerAuth,
) -> impl Responder {
    println!("DEBUG: Petición de retiro recibida.");
    let claims = match decode::<Claims>(
        bearer.token(),
        &DecodingKey::from_secret(env::var("JWT_SECRET").expect("JWT_SECRET must be set").as_ref()),
        &Validation::new(jsonwebtoken::Algorithm::default()),
    ) {
        Ok(token_data) => token_data.claims,
        Err(e) => {
            println!("Error: fallo al decodificar el token {:?}", e);
            return HttpResponse::Unauthorized().finish();
        }
    };

    if claims.role != "accountant" {
        println!("Error: acceso denegado {}", claims.role);
        return HttpResponse::Forbidden().json("Acceso denegado. Solo el contador puede realizar esta acción.");
    }

    if data.amount <= 0.0 {
        println!("Error: Cantidad no valida {}", data.amount);
        return HttpResponse::BadRequest().json("La cantidad debe ser mayor a 0.");
    }
    
    println!("DEBUG: Llamando al servicio de retiro.");

    match accountant_deposit::process_deposit(&pool.db, claims.sub, &data).await {
        Ok(_) => {
        println!("DEBUG: Retiro procesado con éxito.");
        HttpResponse::Ok().json(json!({
            "message": "Depósito realizado con éxito"
        }))
        },
        Err(e) => e,
    }
}
