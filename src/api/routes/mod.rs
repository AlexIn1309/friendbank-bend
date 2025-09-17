use crate::api::handlers::{self, accountant};
use crate::middleware;
use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    // Rutas que no necesitan autenticación
    cfg.service(web::scope("/auth")
        .service(handlers::login::login)
    );

    let auth_middleware = HttpAuthentication::bearer(middleware::jwt_auth::jwt_auth_middleware);

    cfg.service(web::scope("/protected")
        .wrap(auth_middleware.clone())
        .service(handlers::signup::signup)
        .service(handlers::transaction::transfer)
    );
 
    // Rutas solo para el usuario "contador"
    cfg.service(web::scope("/accountant")
        .wrap(auth_middleware) // <-- ¡Correcto!
        .service(accountant::deposit::deposit)
        .service(accountant::withdraw::withdraw)
    );
}
