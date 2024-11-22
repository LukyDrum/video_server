mod storage_object;

use axum::{
    extract::{Path, Request, State},
    response::Html,
    routing::{get, put},
    Router,
};
use futures::StreamExt;

use storage_object::StorageObject;

#[derive(Clone)]
struct ServerState {
    pub obj: StorageObject,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            obj: StorageObject::new("every".to_string()),
        }
    }
}

fn path_to_parts(path: &str) -> (String, String) {
    let mut uri_parts: Vec<&str> = path.split_terminator('/').collect();
    let file = uri_parts.pop().unwrap().to_string();
    let path = uri_parts.join("/");

    (path, file)
}

#[tokio::main]
async fn main() {
    // Init state
    let state = ServerState::new();

    // Build the app using 'route'
    let app = Router::new()
        .route("/", get(root))
        .route("/*filename", put(upload))
        .route("/*filename", get(stream))
        .with_state(state);

    let url = "127.0.0.1:8080";
    // Run app
    let listener = tokio::net::TcpListener::bind(url).await.unwrap();

    println!("\nServer running on {}...", url);
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Html<&'static str> {
    Html("<h1>Video-optimized web server</h1>")
}

async fn upload(State(state): State<ServerState>, request: Request) -> () {
    // Parse the URI into path and file(name)
    let (path, file) = path_to_parts(request.uri().path());

    // Convert body to stream
    let mut stream = request.into_body().into_data_stream();

    // Get new file in storage object
    let file = state.obj.new_file(file);
    // Loop trough stream, wait for bytes and add the bytes to file
    loop {
        if let Some(Ok(bytes)) = stream.next().await {
            // Get write lock for file
            let mut write = file.write().unwrap();
            write.push(bytes);
        } else {
            break;
        }
    }
}

async fn stream(State(state): State<ServerState>, Path(path): Path<String>) -> () {
    let (path, file) = path_to_parts(&path);
    

}
