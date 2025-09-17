// src/middleware/jwt_auth.rs

use actix_web::{
    dev::ServiceRequest,
    Error, HttpMessage, HttpResponse,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env;
 // <-- Importa esto

// El "payload" de nuestro JWT, debe ser público.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32,
    pub exp: u64, 
    pub role: String,
}

// **Este es el único middleware que necesitas para la autenticación**
pub async fn jwt_auth_middleware(req: ServiceRequest, bearer: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = bearer.token();
    
    let secret_key = env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set in .env file");

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::new(jsonwebtoken::Algorithm::default()),
    );

    match claims {
        Ok(token_data) => {
            // Inserta las Claims en las extensiones del request para que los handlers puedan acceder a ellas
            req.extensions_mut().insert(token_data.claims);
            Ok(req)
        },
        Err(_) => {
            let err = actix_web::error::ErrorUnauthorized("Acceso no autorizado");
            Err((err, req))
        }
    }
}

pub async fn verify_accountant_role(
    req: &ServiceRequest,
) -> Result<(), HttpResponse> {
// A more idiomatic way to handle the check
if let Some(claims) = req.extensions().get::<Claims>() {
    if claims.role != "accountant" {
        return Err(HttpResponse::Forbidden().finish());
    }
} else {
    return Err(HttpResponse::Unauthorized().finish());
}

    Ok(())
}
