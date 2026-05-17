use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let addr = env::var("DEBUGPATH_SITE_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:4000".to_owned())
        .parse::<SocketAddr>()
        .expect("DEBUGPATH_SITE_ADDR must be a socket address");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind debugpath-site listener");
    println!("debugpath-site listening on http://{addr}");
    axum::serve(listener, debugpath_site::app(debugpath_site::seeded_site()))
        .await
        .expect("serve debugpath-site");
}
