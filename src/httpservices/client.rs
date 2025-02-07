use bytes::Bytes;
use custom_logger as log;
use http_body_util::{BodyExt, Empty};
use hyper::Request;
use hyper_util::rt::TokioIo;
use tokio::io::{self, AsyncWriteExt as _};
use tokio::net::TcpStream;

pub async fn fetch_url(url: hyper::Uri) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect(addr).await?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            log::error!("Connection failed: {:?}", err);
        }
    });

    let authority = url.authority().unwrap().clone();

    let path = url.path();
    let req = Request::builder()
        .uri(path)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;

    let mut res = sender.send_request(req).await?;

    // Stream the body, writing each chunk to stdout as we get it
    // (instead of buffering and printing at the end).
    while let Some(next) = res.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
            io::stdout().write_all(&chunk).await?;
        }
    }
    Ok(())
}
