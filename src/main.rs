use std::time::Duration;
use std::env;
use axum::response::IntoResponse;
use axum::Json;

use axum::{
  extract::{path, Path, State},
  http::StatusCode,
  routing::{get, patch},Router,
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
struct Response<T> {
    status: bool,
    message: Option<String>,
    data: Option<T>,
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
) -> Result<
    (StatusCode, Json<Response<Vec<Orders>>>),
    (StatusCode, Json<Response<()>>)
    >
     {

    let tr = sqlx::query_as!(Orders, "SELECT * FROM orders ORDER BY id")
    .fetch_all(&pg_pool)
    .await
    .map_err(|_| {
        let error_response = Response {
            status: false,
            message: Some("Error retrieving orders".to_owned()),
            data: None,
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;


    let data = Response {
        status: true,
        message: Some("found orders".to_owned()),
        data: Some(tr)
    };


    Ok((
        StatusCode::OK,
        Json(data),
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
) -> Result<
    (StatusCode, Json<Response<CreateOrdersRow>>),
    (StatusCode, Json<Response<()>>)
>{
    let co = sqlx::query_as!(
    CreateOrdersRow, 
    "INSERT INTO orders (name, coffee_name, size, total) VALUES ($1, $2, $3, $4) RETURNING id", 
    order.name, 
    order.coffee_name,
    order.size, 
    order.total)
    .fetch_one(&pg_pool)
    .await
        .map_err(|_| {
            let error_response = Response {
                status: false,
                message: Some("Error adding order".to_owned()),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let data = Response {
        status: true,
        message: Some("added successfully".to_owned()),
        data: Some(co)
    };

    Ok((
        StatusCode::OK,
        Json(data),
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

) -> Result<
    (StatusCode, Json<Response<CreateOrdersRow>>),
    (StatusCode, Json<Response<()>>)
>{

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
        .map_err(|_| {
            let error_response = Response {
                status: false,
                message: Some("Error updating order".to_owned()),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;
    
        let data = Response {
            status: true,
            message: None,
            data: None
        };
    
    
        Ok((
            StatusCode::OK,
            Json(data),
        ))
}
async fn delete_order(
    Path(id): Path<i32>,
    State(pg_pool): State<PgPool>
) -> Result<
    (StatusCode, Json<Response<CreateOrdersRow>>),
    (StatusCode, Json<Response<()>>)
>{
    sqlx::query!(
        "
        DELETE FROM orders
        WHERE id = $1
         ",
        id
        )
        .execute(&pg_pool)
        .await
        .map_err(|_|{
            let error_response = Response {
                status: false,
                message: Some("Error deleting order".to_owned()),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

        let data = Response {
            status: true,
            message: None,
            data: None
        };
    
    
        Ok((
            StatusCode::OK,
            Json(data),
        ))
}


async fn get_order(
    State(pg_pool): State<PgPool>
) -> Result<(StatusCode, String),(StatusCode, String)>{
    todo!()

}
