use crate::httpservices::server::run_server;
use custom_logger as log;
use std::fs;
use std::path::Path;

mod api;
mod command;
mod error;
mod httpservices;

#[tokio::main]
async fn main() {
    let _ = fs::create_dir_all("logs");

    // setup logging
    log::Logging::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .expect("should initialize");

    // clean up semaphore
    if Path::new("semaphore.pid").exists() {
        fs::remove_file("semaphore.pid").expect("should delete semaphore");
    }
    // Serve over HTTPS, with proper error handling.
    if let Err(e) = run_server().await {
        eprintln!("FAILED: {}", e);
        std::process::exit(1);
    }
}
