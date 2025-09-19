use crate::config::RealmConfig;
use crate::process::ProcessManager;
use anyhow::{Context, Result};
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Clone, Debug)]
struct RouteEntry {
  pattern: String,
  process: String,
  port: u16,
}

pub struct ProxyServer {
  config: RealmConfig,
  process_manager: ProcessManager,
  route_map: Arc<Vec<RouteEntry>>,
}

impl ProxyServer {
  pub fn new(config: RealmConfig, process_manager: ProcessManager) -> Self {
    let route_map = Arc::new(Self::build_route_map(&config));

    Self {
      config,
      process_manager,
      route_map,
    }
  }

  fn build_route_map(config: &RealmConfig) -> Vec<RouteEntry> {
    let mut routes = Vec::new();

    for (process_name, process_config) in &config.processes {
      let port = process_config.port.unwrap_or(3000);

      for route in &process_config.routes {
        routes.push(RouteEntry {
          pattern: route.clone(),
          process: process_name.clone(),
          port,
        });
      }
    }

    routes.sort_by(|a, b| {
      let a_wildcard = a.pattern.contains('*');
      let b_wildcard = b.pattern.contains('*');

      match (a_wildcard, b_wildcard) {
        (false, true) => std::cmp::Ordering::Less,
        (true, false) => std::cmp::Ordering::Greater,
        _ => b.pattern.len().cmp(&a.pattern.len()),
      }
    });

    routes
  }

  pub async fn start(&self) -> Result<()> {
    let addr: SocketAddr = format!("127.0.0.1:{}", self.config.proxy_port)
      .parse()
      .context("Invalid proxy port")?;

    let listener = TcpListener::bind(addr)
      .await
      .context("Failed to bind proxy server")?;

    println!(
      "🚀 Realm proxy server started on http://localhost:{}",
      self.config.proxy_port
    );
    println!("📋 Routes configured:");
    for entry in self.route_map.iter() {
      println!("   {} → {}:{}", entry.pattern, entry.process, entry.port);
    }

    loop {
      let (stream, _) = listener.accept().await?;
      let io = TokioIo::new(stream);

      let route_map = Arc::clone(&self.route_map);
      let process_manager = self.process_manager.clone();

      tokio::task::spawn(async move {
        let route_map = Arc::clone(&route_map);
        let process_manager = process_manager.clone();

        if let Err(err) = http1::Builder::new()
          .serve_connection(
            io,
            service_fn(move |req| {
              let route_map = Arc::clone(&route_map);
              let process_manager = process_manager.clone();

              async move { Self::handle_request(req, route_map, process_manager).await }
            }),
          )
          .await
        {
          eprintln!("Error serving connection: {err:?}");
        }
      });
    }
  }

  async fn handle_request(
    req: Request<Incoming>,
    route_map: Arc<Vec<RouteEntry>>,
    _process_manager: ProcessManager,
  ) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req.uri().path();

    // Health check endpoint
    if path == "/health" {
      return Ok(
        Response::builder()
          .status(StatusCode::OK)
          .header("content-type", "text/plain")
          .body(Full::new(Bytes::from("healthy")))
          .unwrap(),
      );
    }

    // Find matching route
    let target = Self::find_matching_route(path, route_map.as_ref());

    match target {
      Some((process_name, port)) => Self::proxy_request(req, &process_name, port).await,
      None => {
        eprintln!("No route found for path: {path}");
        Ok(
          Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("content-type", "text/html")
            .body(Full::new(Bytes::from(format!(
              "<h1>404 Not Found</h1><p>No route configured for: {path}</p>"
            ))))
            .unwrap(),
        )
      }
    }
  }

  fn find_matching_route(path: &str, route_map: &[RouteEntry]) -> Option<(String, u16)> {
    let mut wildcard_match: Option<(String, u16)> = None;
    let mut default_route: Option<(String, u16)> = None;

    for entry in route_map {
      if !entry.pattern.contains('*') && entry.pattern == path {
        return Some((entry.process.clone(), entry.port));
      }

      if entry.pattern == "/" && default_route.is_none() {
        default_route = Some((entry.process.clone(), entry.port));
      }

      if entry.pattern.ends_with('*') {
        let prefix = entry.pattern.trim_end_matches('*');
        if path.starts_with(prefix) && wildcard_match.is_none() {
          wildcard_match = Some((entry.process.clone(), entry.port));
        }
      }
    }

    wildcard_match.or(default_route)
  }

  async fn proxy_request(
    req: Request<Incoming>,
    process_name: &str,
    port: u16,
  ) -> Result<Response<Full<Bytes>>, Infallible> {
    let target_url = format!("http://127.0.0.1:{port}");

    // Create new request to target
    let uri_string = format!(
      "{}{}",
      target_url,
      req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("")
    );
    let uri = match uri_string.parse::<hyper::Uri>() {
      Ok(uri) => uri,
      Err(e) => {
        eprintln!("Invalid target URI: {e}");
        return Ok(
          Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .body(Full::new(Bytes::from("Invalid target URI")))
            .unwrap(),
        );
      }
    };

    // Build new request
    let mut proxy_req = Request::builder().method(req.method()).uri(uri);

    // Copy headers (except host)
    for (name, value) in req.headers() {
      if name != "host" {
        proxy_req = proxy_req.header(name, value);
      }
    }

    // Set new host header
    proxy_req = proxy_req.header("host", format!("127.0.0.1:{port}"));

    // Get body
    let body = match req.collect().await {
      Ok(collected) => collected.to_bytes(),
      Err(e) => {
        eprintln!("Failed to read request body: {e}");
        return Ok(
          Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Full::new(Bytes::from("Failed to read request body")))
            .unwrap(),
        );
      }
    };

    let proxy_req = match proxy_req.body(Full::new(body)) {
      Ok(req) => req,
      Err(e) => {
        eprintln!("Failed to build proxy request: {e}");
        return Ok(
          Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Full::new(Bytes::from("Failed to build proxy request")))
            .unwrap(),
        );
      }
    };

    // Make the proxied request
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
      .build_http();

    match client.request(proxy_req).await {
      Ok(response) => {
        let (parts, body) = response.into_parts();

        match body.collect().await {
          Ok(collected) => {
            let mut response_builder = Response::builder().status(parts.status);

            // Copy response headers
            for (name, value) in parts.headers {
              response_builder = response_builder.header(name.unwrap(), value);
            }

            // Add CORS headers for development
            response_builder = response_builder
              .header("Access-Control-Allow-Origin", "*")
              .header(
                "Access-Control-Allow-Methods",
                "GET, POST, PUT, DELETE, OPTIONS",
              )
              .header(
                "Access-Control-Allow-Headers",
                "Content-Type, Authorization",
              );

            Ok(
              response_builder
                .body(Full::new(collected.to_bytes()))
                .unwrap(),
            )
          }
          Err(e) => {
            eprintln!("Failed to read response body from {process_name}: {e}");
            Ok(
              Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Full::new(Bytes::from(
                  "Failed to read response from upstream",
                )))
                .unwrap(),
            )
          }
        }
      }
      Err(e) => {
        eprintln!("Failed to proxy request to {process_name} (port {port}): {e}");
        Ok(Response::builder()
                    .status(StatusCode::BAD_GATEWAY)
                    .header("content-type", "text/html")
                    .body(Full::new(Bytes::from(format!(
                        "<h1>502 Bad Gateway</h1><p>Failed to connect to {process_name} (port {port})</p><p>Make sure the process is running.</p>"
                    ))))
                    .unwrap())
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::ProcessConfig;
  use std::collections::HashMap;

  fn build_config(routes: &[(&str, &str, u16, Vec<&str>)]) -> RealmConfig {
    let mut processes = HashMap::new();

    for (name, command, port, route_patterns) in routes {
      processes.insert(
        name.to_string(),
        ProcessConfig {
          command: command.to_string(),
          port: Some(*port),
          routes: route_patterns.iter().map(|r| r.to_string()).collect(),
          working_directory: None,
        },
      );
    }

    RealmConfig {
      env: HashMap::new(),
      env_file: None,
      processes,
      proxy_port: 8000,
    }
  }

  #[test]
  fn chooses_exact_route_over_wildcard() {
    let config = build_config(&[
      ("api_specific", "cmd", 4001, vec!["/api/users"]),
      ("api_wildcard", "cmd", 4002, vec!["/api/*"]),
    ]);

    let routes = ProxyServer::build_route_map(&config);
    let matched = ProxyServer::find_matching_route("/api/users", &routes).unwrap();

    assert_eq!(matched.0, "api_specific");
    assert_eq!(matched.1, 4001);
  }

  #[test]
  fn chooses_most_specific_wildcard() {
    let config = build_config(&[
      ("api_generic", "cmd", 5000, vec!["/api/*"]),
      ("api_users", "cmd", 5001, vec!["/api/users/*"]),
    ]);

    let routes = ProxyServer::build_route_map(&config);
    let matched = ProxyServer::find_matching_route("/api/users/42", &routes).unwrap();

    assert_eq!(matched.0, "api_users");
    assert_eq!(matched.1, 5001);
  }

  #[test]
  fn falls_back_to_root_route() {
    let config = build_config(&[("frontend", "cmd", 3000, vec!["/"])]);

    let routes = ProxyServer::build_route_map(&config);
    let matched = ProxyServer::find_matching_route("/unknown", &routes).unwrap();

    assert_eq!(matched.0, "frontend");
    assert_eq!(matched.1, 3000);
  }

  #[test]
  fn returns_none_when_no_routes_match() {
    let mut config = build_config(&[]);
    config.processes.insert(
      "api".to_string(),
      ProcessConfig {
        command: "cmd".to_string(),
        port: Some(4000),
        routes: vec!["/api".to_string()],
        working_directory: None,
      },
    );

    let routes = ProxyServer::build_route_map(&config);
    assert!(ProxyServer::find_matching_route("/other", &routes).is_none());
  }
}
