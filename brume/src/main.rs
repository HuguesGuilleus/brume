use std::sync::Arc;

use brume::State;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let state = Arc::new(State::new().unwrap());
    let app = io_http::router().with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
