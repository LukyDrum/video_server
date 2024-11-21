use axum::{response::Html, routing::get, Router};

#[tokio::main]
async fn main() {
    // Build the app using 'route'
    let app = Router::new().route("/", get(root));

    let uri = "127.0.0.1:8080";
    // Run app
    let listener = tokio::net::TcpListener::bind(uri).await.unwrap();

    println!("\nServer running on {}...", uri);
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Html<&'static str> {
    Html("<h1>Video-optimized web server</h1>")
}
