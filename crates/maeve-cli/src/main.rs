mod dwell;
mod sse;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let app = sse::router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("maeve CLI listening on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
