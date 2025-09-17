// src/services/accountant/deposit.rs

use sqlx::{MySql, Pool};
use actix_web::HttpResponse;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use crate::models::AccountantData;

pub async fn process_deposit(
    db_pool: &Pool<MySql>,
    accountant_id: i32,
    data: &AccountantData
) -> Result<(), HttpResponse> {
    println!("DEBUG: Iniciando servicio de depósito.");

    let mut transaction = db_pool.begin().await.map_err(|e| {
        println!("ERROR: Fallo al iniciar la transacción: {:?}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    println!("DEBUG: Transacción de base de datos iniciada.");

    // 1. Encontrar el ID de usuario del receptor
    let recipient_user = sqlx::query!(
        "SELECT id FROM users WHERE username = ?",
        data.username
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(|e| {
        if let sqlx::Error::RowNotFound = e {
            println!("ERROR: El usuario no existe.");
            HttpResponse::BadRequest().json("El usuario no existe.")
        } else {
            println!("ERROR: Fallo al buscar el usuario: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    })?;
    
    println!("DEBUG: Usuario receptor encontrado con ID: {}", recipient_user.id);

    let deposit_amount = Decimal::from_f64(data.amount)
        .ok_or_else(|| HttpResponse::BadRequest().json("Monto inválido"))?;

    println!("DEBUG: Monto de depósito convertido a Decimal.");

    // 2. Actualizar el saldo del usuario
    sqlx::query!(
        "UPDATE accounts SET balance = balance + ? WHERE user_id = ?",
        deposit_amount,
        recipient_user.id
    )
    .execute(&mut *transaction)
    .await.map_err(|e| { 
        println!("ERROR: Fallo al actualizar el saldo de la cuenta: {:?}", e);
        HttpResponse::InternalServerError().finish() 
    })?;

    println!("DEBUG: Saldo de la cuenta actualizado con éxito.");

    // 3. Registrar la transacción en la tabla 'transactions'
    sqlx::query!(
        "INSERT INTO transactions (sender_id, recipient_id, amount) VALUES (?, ?, ?)",
        accountant_id,
        recipient_user.id,
        deposit_amount
    )
    .execute(&mut *transaction)
    .await.map_err(|e| {
        println!("ERROR: Fallo al registrar la transacción: {:?}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    println!("DEBUG: Transacción registrada con éxito.");

    // 4. Actualizar el total de dinero en circulación
    sqlx::query!(
        "UPDATE total_supply SET total_amount = total_amount + ?",
        deposit_amount
    )
    .execute(&mut *transaction)
    .await.map_err(|e| { 
        println!("ERROR: Fallo al actualizar el total de dinero en circulación: {:?}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    println!("DEBUG: Total de dinero en circulación actualizado.");

    // 5. Registrar el movimiento en el log de auditoría
    sqlx::query!(
        "INSERT INTO audit_log (amount, type, accountant_user_id) VALUES (?, 'deposit', ?)",
        deposit_amount,
        accountant_id
    )
    .execute(&mut *transaction)
    .await.map_err(|e| {
        println!("ERROR: Fallo al registrar el log de auditoría: {:?}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    println!("DEBUG: Movimiento de auditoría registrado.");

    // 6. Incrementar el contador de transacciones
    sqlx::query!(
        "UPDATE transaction_count SET count = count + 1"
    )
    .execute(&mut *transaction)
    .await.map_err(|e| { 
        println!("ERROR: Fallo al incrementar el contador de transacciones: {:?}", e);
        HttpResponse::InternalServerError().finish() 
    })?;
    
    println!("DEBUG: Contador de transacciones incrementado.");

    // 7. Confirmar la transacción
    transaction.commit().await.map_err(|e| {
        println!("ERROR: Fallo al confirmar la transacción: {:?}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    println!("DEBUG: Transacción completada con éxito.");

    Ok(())
}
