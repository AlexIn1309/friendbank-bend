// src/main.rs

use actix_web::{get, web, App, HttpServer, Responder};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};
use dotenv::dotenv;
use std::env;

mod models;
mod api;
mod middleware;
mod services;

#[derive(Clone)]
struct AppState {
    db: Pool<MySql>,
}

#[get("/")]
async fn index() -> impl Responder {
    "Biendvenido a FriendBank"
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    println!("DATABASE_URL: {:?}", env::var("DATABASE_URL"));

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    let db_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create MySql pool.");

    println!("Connected to the database succesfully!");

    let app_state = AppState { db: db_pool.clone() };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(index)
            .configure(api::routes::config_routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
