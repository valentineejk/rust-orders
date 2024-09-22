use std::time::Duration;
use std::env;
use axum::Json;

use axum::{
  extract::{path, Path, State},
  http::StatusCode,
  routing::{get, patch},
  Json, Router,
};
use axum::http::header::IF_MATCH;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;


#[tokio::main]
async fn main() {

    //ENV SETUP
    // Only load `.env` file in development

    if env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()) == "development" {
        dotenvy::dotenv().ok();  // Load `.env` in development environment
        println!("Loaded .env file");
    } else {
        println!("Loading prod env");
    }


    //VARIABLES FROM ENV
    let url = std::env::var("DATABASE_URL")
        .expect("database url is empty or not provided");
        
    //DB POOL
    let db = PgPoolOptions::new()
    .max_connections(16)
    .connect(&url)
    .await
    .expect("cannot connect to database");

    //TCP
    let lis = TcpListener::bind("0.0.0.0:3000".to_owned())
    .await
    .expect("could not create tcp listener");

    println!("listening on {}", lis.local_addr().unwrap());

    //ROUTES
   let r = Router::new()
    .route("/", get(|| async {"MAY THE FORCE BE WITH YOU"}))
    .route("/orders", get(get_orders).post(add_order))
    .route("/orders/:id", get(get_order).put(update_order).delete(delete_order))
    .with_state(db);

    //SERVER
    axum::serve(lis, r).await.expect("error starting server");
    println!("Hello, world!");
}


#[derive(Serialize)]
struct Orders {
    id: Option<i32>,
    name: Option<String>,
    coffee_name: Option<String>,
    size: Option<String>,
    total: Option<i32>,
}


async fn get_orders(
    State(pg_pool): State<PgPool>
) -> Result<(StatusCode, String),(StatusCode, String)>{
    let tr = sqlx::query_as!(Orders, "SELECT * FROM orders ORDER BY id")
    .fetch_all(&pg_pool)
    .await
    .map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR, 
        json!({"status": false, "message": "orders retrieved"}).to_string(),
    ))?;



    Ok((
        StatusCode::OK,
        Json(json!({"status": true, "data": tr}).to_string()),
    ))
}

#[derive(Deserialize)]
struct CreateOrdersReq {
    name: String,
    coffee_name: String,
    size: String,
    total: i32,
}

#[derive(sqlx::FromRow, Serialize)]
struct CreateOrdersRow {
    id: i32
}

async fn add_order(
    State(pg_pool): State<PgPool>,
    Json(order): Json<CreateOrdersReq>,
) -> Result<(StatusCode, String),(StatusCode, String)>{
    let co = sqlx::query_as!(
    CreateOrdersRow, 
    "INSERT INTO orders (name, coffee_name, size, total) VALUES ($1, $2, $3, $4) RETURNING id", 
    order.name, 
    order.coffee_name,
    order.size, 
    order.total)
    .fetch_one(&pg_pool)
    .await
    .map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR, 
        json!({"status": false, "message": "error adding order to db"}).to_string(),
    ))?;

    Ok((
        StatusCode::CREATED,
        Json(json!({"status": true, "data": co}).to_string()),
    ))
}


#[derive(Deserialize)]
struct UpdateOrdersReq {
    name: Option<String>,
    coffee_name: Option<String>,
    size: Option<String>,
    total: Option<i32>,
}


async fn update_order(
    State(pg_pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(order): Json<UpdateOrdersReq>,

) -> Result<(StatusCode, String),(StatusCode, String)>{

    let mut q = "UPDATE orders SET id = $1".to_owned();

    let mut i = 2;

    if order.name.is_some() {
        q.push_str(&format!(", name = ${i}"));
        i = i + 1;
    };

    if order.coffee_name.is_some() {
        q.push_str(&format!(", coffee_name = ${i}"));
        i = i + 1;
    };

    if order.size.is_some() {
        q.push_str(&format!(", size = ${i}"));
        i = i + 1;
    };

    if order.total.is_some() {
        q.push_str(&format!(", total = ${i}"));
    };

    q.push_str(&format!(" WHERE id = $1"));

    let mut s = sqlx::query(&q).bind(id);

    if order.name.is_some() {
        s = s.bind(order.name);
    }

    if order.coffee_name.is_some() {
        s = s.bind(order.coffee_name);
    }

    if order.size.is_some() {
        s = s.bind(order.size);
    }

    if order.total.is_some() {
        s = s.bind(order.total);
    }


    s.execute(&pg_pool)
        .await
        .map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR, 
            json!({"status": false, "message": "error updating order"}).to_string(),
        ))?;
    
        Ok((
            StatusCode::OK,
            Json(json!({"status": true}).to_string()),
        ))
}
async fn delete_order(
    Path(id): Path<i32>,
    State(pg_pool): State<PgPool>
) -> Result<(StatusCode, String),(StatusCode, String)>{
    sqlx::query!(
        "
        DELETE FROM orders
        WHERE id = $1
         ",
        id
        )
        .execute(&pg_pool)
        .await
        .map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR,
                     json!({"status": false, "message": "error updating order"}).to_string(),
        ))?;

    Ok((
        StatusCode::OK,
        Json(json!({"status": true}).to_string()),
    ))
}


async fn get_order(
    State(pg_pool): State<PgPool>
) -> Result<(StatusCode, String),(StatusCode, String)>{
    todo!()

}
