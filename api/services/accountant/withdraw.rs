// src/services/accountant/withdraw.rs

use sqlx::{MySql, Pool};
use actix_web::HttpResponse;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use crate::models::AccountantData;

pub async fn process_withdrawal(
    db_pool: &Pool<MySql>,
    accountant_id: i32,
    data: &AccountantData
) -> Result<(), HttpResponse> {
    let mut transaction = db_pool.begin().await.map_err(|_| {
        HttpResponse::InternalServerError().finish()
    })?;

    // 1. Encontrar el ID de usuario del remitente
    let sender_user = sqlx::query!(
        "SELECT id FROM users WHERE username = ?",
        data.username
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(|e| {
        if let sqlx::Error::RowNotFound = e {
            HttpResponse::BadRequest().json("El usuario no existe.")
        } else {
            HttpResponse::InternalServerError().finish()
        }
    })?;

    let withdrawal_amount = Decimal::from_f64(data.amount)
        .ok_or_else(|| HttpResponse::BadRequest().json("Monto inválido"))?;

    // 2. Obtener el saldo de la cuenta
    let sender_account = sqlx::query!(
        "SELECT balance FROM accounts WHERE user_id = ?",
        sender_user.id
    )
    .fetch_one(&mut *transaction)
    .await.map_err(|_| {
        HttpResponse::BadRequest().json("No se encontró la cuenta del usuario.")
    })?;

    // 3. Validar que la cuenta tenga fondos suficientes
    if sender_account.balance < withdrawal_amount {
        return Err(HttpResponse::BadRequest().json("Fondos insuficientes en la cuenta del usuario."));
    }

    // 4. Actualizar el saldo del usuario
    sqlx::query!(
        "UPDATE accounts SET balance = balance - ? WHERE user_id = ?",
        withdrawal_amount,
        sender_user.id
    )
    .execute(&mut *transaction)
    .await.map_err(|_| { HttpResponse::InternalServerError().finish() })?;

    // 5. Registrar la transacción en la tabla 'transactions'
    sqlx::query!(
        "INSERT INTO transactions (sender_id, recipient_id, amount) VALUES (?, ?, ?)", // recipient_id es NULL para un retiro
        sender_user.id,
        accountant_id,
        withdrawal_amount
    )
    .execute(&mut *transaction)
    .await.map_err(|_| { HttpResponse::InternalServerError().finish() })?;

    // 6. Actualizar el total de dinero en circulación
    sqlx::query!(
        "UPDATE total_supply SET total_amount = total_amount - ?",
        withdrawal_amount
    )
    .execute(&mut *transaction)
    .await.map_err(|_| { HttpResponse::InternalServerError().finish() })?;

    // 7. Registrar el movimiento en el log de auditoría
    sqlx::query!(
        "INSERT INTO audit_log (amount, type, accountant_user_id) VALUES (?, 'withdrawal', ?)",
        withdrawal_amount,
        accountant_id
    )
    .execute(&mut *transaction)
    .await.map_err(|_| { HttpResponse::InternalServerError().finish() })?;

    // 8. Incrementar el contador de transacciones
    sqlx::query!(
        "UPDATE transaction_count SET count = count + 1"
    )
    .execute(&mut *transaction)
    .await.map_err(|_| { HttpResponse::InternalServerError().finish() })?;

    // 9. Confirmar la transacción
    transaction.commit().await.map_err(|_| {
        HttpResponse::InternalServerError().finish()
    })?;

    Ok(())
}
