use sqlx::{MySql, Pool};
use crate::models::{User, UserData};
use bcrypt::verify;
use actix_web::HttpResponse;
pub async fn verify_login(db_pool: &Pool<MySql>, user_data: &UserData) -> Result<User, HttpResponse> {
    let user = match sqlx::query_as!(
        User,
        "SELECT id, username, password_hash, role FROM users WHERE username = ?",
        user_data.username
    )
    .fetch_one(db_pool)
    .await {
        Ok(u) => u,
        Err(sqlx::Error::RowNotFound) => {
            return Err(HttpResponse::Unauthorized().json("Credenciales incorrectas"));
        }
        Err(_) => return Err(HttpResponse::InternalServerError().finish()),
    };

    let is_password_valid = match verify(&user_data.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(_) => false,
    };

    if !is_password_valid {
        return Err(HttpResponse::Unauthorized().json("Credenciales incorrectas"));
    }

    Ok(user)
}
