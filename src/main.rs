// Pingora 0.8.1 — Load Balancer para a Rinha
//
// Configuração via env vars:
//   LISTEN (padrão: 0.0.0.0:9999)
//   UPSTREAMS (padrão: api1:8080,api2:8081)
//   HEALTH_CHECK_INTERVAL (padrão: 5)

use async_trait::async_trait;
use pingora::lb::selection::RoundRobin;
use pingora::lb::{health_check::TcpHealthCheck, LoadBalancer};
use pingora::prelude::*;
use std::sync::Arc;
use std::time::Duration;

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
        let health_check_interval = std::env::var("HEALTH_CHECK_INTERVAL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);
        Config { listen, upstreams, health_check_interval }
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
        let mut peer = Box::new(HttpPeer::new(upstream, false, String::new()));
        peer.options.connection_timeout = Some(Duration::from_secs(5));
        peer.options.read_timeout = Some(Duration::from_secs(10));
        peer.options.idle_timeout = Some(Duration::from_secs(60));
        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        upstream_request.append_header("Connection", "keep-alive").unwrap();
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let config = Config::from_env();

    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    let addrs: Vec<&str> = config.upstreams.iter().map(|s| s.as_str()).collect();
    let mut upstreams = LoadBalancer::try_from_iter(addrs).unwrap();

    let hc = TcpHealthCheck::new();
    upstreams.set_health_check(hc);
    upstreams.health_check_frequency =
        Some(Duration::from_secs(config.health_check_interval));

    let bg = background_service("health check", upstreams);
    let upstreams = bg.task();

    let mut proxy = http_proxy_service(&server.configuration, LB(upstreams));
    proxy.add_tcp(&config.listen);

    server.add_service(bg);
    server.add_service(proxy);

    println!("Pingora LB listening on http://{}", config.listen);
    println!("Upstreams: {:?}", config.upstreams);
    server.run_forever();
}
