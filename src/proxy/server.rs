use crate::config::RealmConfig;
use crate::process::ProcessManager;
use anyhow::{Context, Result};
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub struct ProxyServer {
  config: RealmConfig,
  process_manager: ProcessManager,
  route_map: HashMap<String, (String, u16)>, // route -> (process_name, port)
}

impl ProxyServer {
  pub fn new(config: RealmConfig, process_manager: ProcessManager) -> Self {
    let route_map = Self::build_route_map(&config);

    Self {
      config,
      process_manager,
      route_map,
    }
  }

  fn build_route_map(config: &RealmConfig) -> HashMap<String, (String, u16)> {
    let mut route_map = HashMap::new();

    for (process_name, process_config) in &config.processes {
      let port = process_config.port.unwrap_or(3000);

      for route in &process_config.routes {
        route_map.insert(route.clone(), (process_name.clone(), port));
      }
    }

    // Sort routes by specificity (longer/more specific routes first)
    let mut routes: Vec<_> = route_map.keys().cloned().collect();
    routes.sort_by(|a, b| {
      // Exact matches come before wildcard matches
      let a_wildcard = a.contains('*');
      let b_wildcard = b.contains('*');

      match (a_wildcard, b_wildcard) {
        (false, true) => std::cmp::Ordering::Less,
        (true, false) => std::cmp::Ordering::Greater,
        _ => b.len().cmp(&a.len()), // Longer routes first
      }
    });

    // Rebuild map with sorted order
    let mut sorted_map = HashMap::new();
    for route in routes {
      if let Some(value) = route_map.get(&route) {
        sorted_map.insert(route, value.clone());
      }
    }

    sorted_map
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
    for (route, (process, port)) in &self.route_map {
      println!("   {route} → {process}:{port}");
    }

    loop {
      let (stream, _) = listener.accept().await?;
      let io = TokioIo::new(stream);

      let route_map = self.route_map.clone();
      let process_manager = self.process_manager.clone();

      tokio::task::spawn(async move {
        if let Err(err) = http1::Builder::new()
          .serve_connection(
            io,
            service_fn(move |req| {
              Self::handle_request(req, route_map.clone(), process_manager.clone())
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
    route_map: HashMap<String, (String, u16)>,
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
    let target = Self::find_matching_route(path, &route_map);

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

  fn find_matching_route(
    path: &str,
    route_map: &HashMap<String, (String, u16)>,
  ) -> Option<(String, u16)> {
    // Try exact match first
    if let Some((process, port)) = route_map.get(path) {
      return Some((process.clone(), *port));
    }

    // Try prefix matching for wildcard routes
    for (route, (process, port)) in route_map {
      if route.ends_with("*") {
        let prefix = &route[..route.len() - 1];
        if path.starts_with(prefix) {
          return Some((process.clone(), *port));
        }
      }
    }

    // Default to root route if exists
    if let Some((process, port)) = route_map.get("/") {
      return Some((process.clone(), *port));
    }

    None
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
        eprintln!(
          "Failed to proxy request to {process_name} (port {port}): {e}"
        );
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
