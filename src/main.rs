mod file;
mod storage_object;

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{delete, get, put},
    Router,
};

use futures::StreamExt;
use storage_object::StorageObject;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct ServerState {
    pub obj: StorageObject,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            obj: StorageObject::new(),
        }
    }
}

#[tokio::main]
async fn main() {
    // Init state
    let state = ServerState::new();

    let cors_layer = CorsLayer::new().allow_origin(Any);

    // Build the app using 'route'
    let app = Router::new()
        .route("/", get(root))
        .route("/*filename", put(upload))
        .route("/*filename", get(stream))
        .route("/*filename", delete(file_delete))
        .layer(ServiceBuilder::new().layer(cors_layer))
        .with_state(state);

    let url = "127.0.0.1:8080";
    // Run app
    let listener = tokio::net::TcpListener::bind(url).await.unwrap();

    println!("\nServer running on {}...\n", url);
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Html<&'static str> {
    Html("<h1>Video-optimized web server</h1>")
}

async fn upload(State(state): State<ServerState>, request: Request) -> impl IntoResponse {
    // Get the path
    let path = request.uri().path().to_string();

    // Debug print
    println!("Uploading: {}", path);
    // Convert body to stream
    let mut stream = request.into_body().into_data_stream();

    // Get new file in storage object
    let file = state.obj.new_file(path);

    // Loop trough stream, wait for bytes and add the bytes to file
    while let Some(Ok(bytes)) = stream.next().await {
        // Get write lock for file
        let mut write = file.write().unwrap();
        write.push(bytes);
    }

    let mut fl = file.write().unwrap();
    fl.set_as_complete();

    StatusCode::CREATED
}

async fn stream(
    State(state): State<ServerState>,
    request: Request,
) -> (StatusCode, Body) {
    let path = request.uri().path().to_string();
    match state.obj.get_filestream(&path) {
        Some(stream) => (StatusCode::OK, Body::from_stream(stream)),
        None => (StatusCode::NOT_FOUND, Body::from("File not found")),
    }
}

async fn file_delete(State(state): State<ServerState>, request: Request) -> impl IntoResponse {
    let path = request.uri().path().to_string();
    match state.obj.delete_file(&path) {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::NOT_FOUND
    }
}
