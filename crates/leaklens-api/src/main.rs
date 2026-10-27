use std::{env, net::SocketAddr};

use leaklens_api::router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_owned())
        .parse::<u16>()?;
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(address).await?;
    let origin = env::var("LEAKLENS_ALLOWED_ORIGIN").ok();
    println!("LeakLens read-only API listening on {address}");
    axum::serve(listener, router(origin)).await?;
    Ok(())
}
