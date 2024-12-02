use crate::httpservices::server::run_server;

mod api;
mod httpservices;
mod remote;

#[tokio::main]
async fn main() {
    // Serve over HTTPS, with proper error handling.
    if let Err(e) = run_server().await {
        eprintln!("FAILED: {}", e);
        std::process::exit(1);
    }
}
