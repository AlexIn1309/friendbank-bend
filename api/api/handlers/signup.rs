use crate::models::UserData;
use actix_web::{post, web, HttpResponse, Responder};


#[post("/signup")]
pub async fn signup(
    pool: web::Data<crate::AppState>,
    user_data: web::Json<UserData>,
) -> impl Responder {
    let mut transaction = match pool.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let hashed_password = match bcrypt::hash(&user_data.password, 10) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let user_id: i32 = match sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES (?, ?)",
        user_data.username,
        hashed_password
    )
    .execute(&mut *transaction)
    .await {
        Ok(result) => result.last_insert_id() as i32,
        Err(_) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    match sqlx::query!(
        "INSERT INTO accounts (user_id, balance) VALUES (?, 0)",
        user_id
    )
    .execute(&mut *transaction)
    .await {
        Ok(_) => {
            // 5. Si todo es exitoso, confirmar la transacción
            if transaction.commit().await.is_ok() {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Usuario creado exitosamente!"
                }))
            } else {
                HttpResponse::InternalServerError().finish()
            }
        },
        Err(_) => {
            // 6. Si falla la inserción de la cuenta, la transacción se revierte
            HttpResponse::InternalServerError().finish()
        }
    }
}
