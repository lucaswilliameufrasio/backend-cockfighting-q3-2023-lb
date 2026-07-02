// Pingora 0.8.1 — Load Balancer completo para a Rinha
//
// Funcionalidades extras:
//   - configuração via env vars
//   - header de upstream personalizado
//   - logging de cada request
//   - graceful shutdown

use async_trait::async_trait;
use pingora::lb::selection::RoundRobin;
use pingora::lb::{health_check::TcpHealthCheck, LoadBalancer};
use pingora::prelude::*;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

struct Config {
    listen: String,
    upstreams: Vec<String>,
    health_check_interval: u64,
}

impl Config {
    fn from_env() -> Self {
        let listen = std::env::var("LISTEN").unwrap_or_else(|_| "0.0.0.0:9999".into());
        let upstreams_str =
            std::env::var("UPSTREAMS").unwrap_or_else(|_| "api1:8080,api2:8081".into());
        let upstreams: Vec<String> =
            upstreams_str.split(',').map(|s| s.trim().to_string()).collect();
        let health_check_interval: u64 = std::env::var("HEALTH_CHECK_INTERVAL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        Config {
            listen,
            upstreams,
            health_check_interval,
        }
    }
}

// ---------------------------------------------------------------------------
// Proxy
// ---------------------------------------------------------------------------

struct LB(Arc<LoadBalancer<RoundRobin>>);

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let upstream = self.0.select(b"", 256).unwrap();
        let peer = Box::new(HttpPeer::new(upstream, false, String::new()));
        Ok(peer)
    }

    /// Loga o request (opcional, útil para debugging)
    async fn logging(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
        e: Option<&pingora::Error>,
    ) {
        let req = session.req_header();
        if let Some(err) = e {
            eprintln!(
                "ERROR {} {} -> {} (via {})",
                req.method,
                req.uri.path_and_query.map(|p| p.as_str()).unwrap_or("/"),
                err,
                session
                    .upstream_peer()
                    .map(|p| p.address.to_string())
                    .unwrap_or_default()
            );
        } else {
            println!(
                "{} {} -> {}",
                req.method,
                req.uri.path_and_query.map(|p| p.as_str()).unwrap_or("/"),
                session
                    .upstream_peer()
                    .map(|p| p.address.to_string())
                    .unwrap_or_default(),
            );
        }
    }

    /// Adiciona header personalizado no request para o upstream
    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        upstream_request
            .append_header("X-Forwarded-By", "pingora")
            .unwrap();
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let cfg = Config::from_env();

    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    // upstreams
    let addrs: Vec<&str> = cfg.upstreams.iter().map(|s| s.as_str()).collect();
    let mut upstreams = LoadBalancer::try_from_iter(addrs).unwrap();

    // health check
    let hc = TcpHealthCheck::new();
    upstreams.set_health_check(hc);
    upstreams.health_check_frequency =
        Some(std::time::Duration::from_secs(cfg.health_check_interval));

    let bg = background_service("health check", upstreams);
    let upstreams = bg.task();

    // proxy
    let mut proxy = http_proxy_service(&server.configuration, LB(upstreams));
    proxy.add_tcp(&cfg.listen);

    server.add_service(bg);
    server.add_service(proxy);

    println!("Pingora LB listening on http://{}", cfg.listen);
    println!("Upstreams: {:?}", cfg.upstreams);
    server.run_forever();
}
