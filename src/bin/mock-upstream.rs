// Mock upstream HTTP server for integration tests.
// Responde com identificação do backend e ecoa o path.
//
// Uso: cargo run --bin mock-upstream -- 8080

use std::net::SocketAddr;

use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

async fn handle(req: Request<Incoming>, port: u16) -> Result<Response<String>, hyper::Error> {
    let path = req
        .uri()
        .path_and_query()
        .map(|p| p.as_str())
        .unwrap_or("/");

    match path {
        "/health-check" => Ok({
            let body = format!("healthy:{}", port);
            let mut resp = Response::new(body);
            *resp.status_mut() = StatusCode::OK;
            resp.headers_mut()
                .insert("content-type", "text/plain".parse().unwrap());
            resp
        }),
        _ => Ok({
            let body = format!("upstream:{} path:{}", port, path);
            let mut resp = Response::new(body);
            *resp.status_mut() = StatusCode::OK;
            resp.headers_mut()
                .insert("content-type", "text/plain".parse().unwrap());
            resp
        }),
    }
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);

    let addr: SocketAddr = ([0, 0, 0, 0], port).into();
    let listener = TcpListener::bind(addr).await.unwrap();
    eprintln!("Mock upstream listening on {}", addr);

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);

        tokio::spawn(async move {
            let svc = service_fn(move |req| handle(req, port));
            let _ = http1::Builder::new()
                .serve_connection(io, svc)
                .await;
        });
    }
}
