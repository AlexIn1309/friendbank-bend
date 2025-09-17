use crate::services::transaction_service;
use actix_web::{post, web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::env;

// Importa el servicio y la estructura de datos
use crate::models::TransactionData;
use crate::middleware::jwt_auth::Claims;

#[post("/transfer")]
pub async fn transfer(
    pool: web::Data<crate::AppState>,
    transaction_data: web::Json<TransactionData>,
    bearer: BearerAuth,
) -> impl Responder {
    println!("DEBUG: Petición de transferencia recibida.");

    let claims = match decode::<Claims>(
        bearer.token(),
        &DecodingKey::from_secret(env::var("JWT_SECRET").expect("JWT_SECRET must be set").as_ref()),
        &Validation::new(jsonwebtoken::Algorithm::default()),
    ) {
        Ok(token_data) => token_data.claims,
        Err(e) => {
            println!("ERROR: Fallo al decodificar el token JWT: {:?}", e);
            return HttpResponse::Unauthorized().finish();
        }
    };

    println!("DEBUG: Token JWT decodificado con éxito para el usuario ID: {}", claims.sub);

    if transaction_data.amount <= 0.0 {
        println!("ERROR: Monto de transferencia no válido: {}", transaction_data.amount);
        return HttpResponse::BadRequest().json("La cantidad debe ser mayor a 0");
    }

    println!("DEBUG: Llamando al servicio de procesamiento de transferencia.");

    // Llama al servicio para procesar la lógica de negocio
    match transaction_service::process_transfer(&pool.db, claims.sub, &transaction_data).await {
        Ok(_) => {
            println!("DEBUG: El servicio de transferencia se completó con éxito.");
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Transferencia realizada con éxito"
            }))
        },
        Err(e) => {
            println!("ERROR: El servicio de transferencia falló.");
            e
        },
    }
}
