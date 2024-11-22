mod storage_object;

use axum::{
    extract::{Request, State},
    response::Html,
    routing::{get, put},
    Router,
};
use futures::StreamExt;

use storage_object::StorageObject;


#[derive(Clone)]
struct ServerState {
    pub obj: StorageObject
}

impl ServerState {
    pub fn new() -> Self {
        ServerState { obj: StorageObject::new("every".to_string()) }
    }
}

#[tokio::main]
async fn main() {
    // Init state
    let state = ServerState::new();

    // Build the app using 'route'
    let app = Router::new()
        .route("/", get(root))
        .route("/*filename", put(upload))
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
    let mut uri_parts: Vec<&str> = request.uri().path().split_terminator('/').collect();
    let file = uri_parts.pop().unwrap().to_string();
    let path = uri_parts.join("/");
    
    // Convert body to stream
    let mut stream = request.into_body().into_data_stream();

    // Get new (last) segment in storage object
    let segment = state.obj.new_segment();
    // Loop trough stream, wait for bytes and add the bytes to last segment
    loop {
        if let Some(Ok(bytes)) = stream.next().await {
            // Update metadata (eg. .mpd)
            if file.ends_with(".mpd") {
                state.obj.update_meta(bytes);
            }
            // Handle chunks
            else {
                // Get write lock for segment
                let mut write = segment.write().unwrap();
                write.push(bytes);
            }

        } else {
            break;
        }
    }
}
