use sqlx::{MySql, Pool};
use actix_web::HttpResponse;
use crate::models::{Account, TransactionData};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

pub async fn process_transfer(
    db_pool: &Pool<MySql>,
    sender_user_id: i32,
    transaction_data: &TransactionData
) -> Result<(), HttpResponse> {
    println!("DEBUG: Iniciando la transacción de base de datos.");

    // Start a database transaction
    let mut transaction = db_pool.begin().await.map_err(|e| {
        println!("ERROR: Fallo al iniciar la transacción: {:?}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    println!("DEBUG: Transacción iniciada con éxito.");

    // 1. Get the account IDs and balances
    let sender_account = sqlx::query_as!(
        Account,
        "SELECT id, user_id, balance FROM accounts WHERE user_id = ?",
        sender_user_id
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(|e| {
        println!("ERROR: Fallo al obtener la cuenta del emisor: {:?}", e);
        HttpResponse::BadRequest().json("No se encontró la cuenta del emisor.")
    })?;

    println!("DEBUG: Cuenta del emisor encontrada: {:?}", sender_account.id);

    let recipient_user = sqlx::query!(
        "SELECT id FROM users WHERE username = ?",
        transaction_data.recipient_username
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(|e| {
        if let sqlx::Error::RowNotFound = e {
            HttpResponse::BadRequest().json("El usuario receptor no existe")
        } else {
            println!("ERROR: Fallo al obtener el usuario receptor: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    })?;

    println!("DEBUG: Usuario receptor encontrado: {:?}", recipient_user.id);

    let recipient_account = sqlx::query_as!(
        Account,
        "SELECT id, user_id, balance FROM accounts WHERE user_id = ?",
        recipient_user.id
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(|e| {
        println!("ERROR: Fallo al obtener la cuenta del receptor: {:?}", e);
        HttpResponse::InternalServerError().json("No se encontró la cuenta del receptor.")
    })?;

    println!("DEBUG: Cuenta del receptor encontrada: {:?}", recipient_account.id);

    // 2. Validate the balance
    let transaction_amount = Decimal::from_f64(transaction_data.amount)
        .ok_or_else(|| HttpResponse::BadRequest().json("Monto inválido"))?;

    println!("DEBUG: El monto de la transacción es: {:?}", transaction_amount);

    if sender_account.balance < transaction_amount {
        println!("DEBUG: Fondos insuficientes. Saldo actual: {:?}, Monto: {:?}", sender_account.balance, transaction_amount);
        return Err(HttpResponse::BadRequest().json("Fondos insuficientes"));
    }
    
    // 3. Update balances and record the transaction
    println!("DEBUG: Actualizando el saldo del emisor.");
    sqlx::query!(
        "UPDATE accounts SET balance = balance - ? WHERE id = ?",
        transaction_amount,
        sender_account.id
    )
    .execute(&mut *transaction)
    .await.map_err(|e| { 
        println!("ERROR: Fallo al actualizar la cuenta del emisor: {:?}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    println!("DEBUG: Actualizando el saldo del receptor.");
    sqlx::query!(
        "UPDATE accounts SET balance = balance + ? WHERE id = ?",
        transaction_amount,
        recipient_account.id
    )
    .execute(&mut *transaction)
    .await.map_err(|e| {
        println!("ERROR: Fallo al actualizar la cuenta del receptor: {:?}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    println!("DEBUG: Registrando la transacción en la base de datos.");
    sqlx::query!(
        "INSERT INTO transactions (sender_id, recipient_id, amount) VALUES (?, ?, ?)",
        sender_account.id,
        recipient_account.id,
        transaction_amount
    )
    .execute(&mut *transaction)
    .await.map_err(|e| { 
        println!("ERROR: Fallo al insertar la transacción: {:?}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    // 4. Commit the transaction
    println!("DEBUG: Intentando confirmar la transacción.");
    transaction.commit().await.map_err(|e| {
        println!("ERROR: Fallo al confirmar la transacción: {:?}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    
    println!("DEBUG: ¡Transacción confirmada con éxito!");

    Ok(())
}
