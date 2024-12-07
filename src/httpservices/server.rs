use crate::api::schema::TaskExecute;
use crate::command::process::{local_execute, remote_execute};
use http::{Method, Request, Response, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::str;
use std::sync::Arc;
use std::{env, fs, io};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

fn error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // First parameter is port number (optional, defaults to 1337)
    let port = match env::args().nth(1) {
        Some(ref p) => p.parse()?,
        None => 1337,
    };
    let addr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), port);

    // Load public certificate.
    let certs = load_certs("certs/ssl.cert")?;
    // Load private key.
    let key = load_private_key("certs/ssl.key")?;

    println!("starting to serve on https://{}", addr);

    // Create a TCP listener via tokio.
    let incoming = TcpListener::bind(&addr).await?;

    // Build TLS configuration.
    let mut server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| error(e.to_string()))?;
    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec(), b"http/1.0".to_vec()];
    let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));

    let service = service_fn(process);

    loop {
        let (tcp_stream, _remote_addr) = incoming.accept().await?;

        let tls_acceptor = tls_acceptor.clone();
        tokio::spawn(async move {
            let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                Ok(tls_stream) => tls_stream,
                Err(err) => {
                    eprintln!("failed to perform tls handshake: {err:#}");
                    return;
                }
            };
            if let Err(err) = Builder::new(TokioExecutor::new())
                .serve_connection(TokioIo::new(tls_stream), service)
                .await
            {
                eprintln!("failed to serve connection: {err:#}");
            }
        });
    }
}

// Custom service, handling two different routes and a
// catch-all 404 responder.
async fn process(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let mut response = Response::new(Full::default());
    match (req.method(), req.uri().path()) {
        // help route.
        (&Method::GET, "/") => {
            *response.body_mut() = Full::from("use POST to '/process' endpoint\n");
        }
        // service route.
        (&Method::POST, "/process") => {
            // simple file based semaphore - dropped when callback is completed
            if Path::new("semaphore.pid").exists() {
                *response.body_mut() = Full::from("process has not completed yet");
            } else {
                let req_data = req.into_body().collect().await?.to_bytes();
                let s = String::from_utf8(req_data.to_vec()).unwrap();
                let task_exec: TaskExecute = serde_json::from_str(&s).unwrap();
                *response.body_mut() = Full::from("process command sent");
                // acquire
                fs::write("semaphore.pid", "process").expect("should write semaphore");
                for node in task_exec.spec.nodes.iter().to_owned() {
                    if node.name == "localhost" {
                        let node_params = node.clone();
                        tokio::spawn(async move {
                            local_execute(&node_params).await;
                        });
                    } else {
                        let node_params = node.clone();
                        tokio::spawn(async move {
                            let _ = remote_execute(&node_params).await;
                        });
                    }
                }
            }
        }
        // not found
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Ok(response)
}

// load public certificate from file.
fn load_certs(filename: &str) -> io::Result<Vec<CertificateDer<'static>>> {
    let certfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader).collect()
}

// load private key from file.
fn load_private_key(filename: &str) -> io::Result<PrivateKeyDer<'static>> {
    let keyfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(keyfile);
    rustls_pemfile::private_key(&mut reader).map(|key| key.unwrap())
}
