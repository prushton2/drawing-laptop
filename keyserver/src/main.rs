use std::{collections::HashMap, sync::Arc};

use axum::{
    Router, extract::{Path, State}, http::StatusCode, routing::{delete, get, put},
};
use tokio::sync::RwLock;

type Store = Arc<RwLock<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    let store: Store = Arc::new(RwLock::new(HashMap::new()));

    let app = Router::new()
        .route("/keys/{name}", put(set_key))
        .route("/keys/{name}", get(get_key))
        .route("/keys/{name}", delete(delete_key))
        .with_state(store);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("keyserver listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn get_key(Path(name): Path<String>, State(store): State<Store>) -> Result<String, StatusCode> {
    let store = store.read().await;
    store.get(&name).cloned().ok_or(StatusCode::NOT_FOUND)
}

async fn set_key(Path(name): Path<String>, State(store): State<Store>, body: String) -> StatusCode {
    let mut store = store.write().await;
    store.insert(name, body.trim().to_string());
    StatusCode::NO_CONTENT
}

async fn delete_key(Path(name): Path<String>, State(store): State<Store>) -> StatusCode {
    let mut store = store.write().await;
    match store.remove(&name) {
        Some(_) => StatusCode::NO_CONTENT,
        None => StatusCode::NOT_FOUND,
    }
}
