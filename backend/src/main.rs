mod clash;
mod log;

use std::convert::Infallible;
use std::sync::atomic::{AtomicU64, Ordering};

use http_body_util::Full;
use hyper::body::{Body, Bytes, Incoming};
use hyper::{Request, Response};
use serde_json::json;

/// Numbers the requests since the start, so the request line and what a handler logs about it can be
/// matched by `request_id`.
static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);

async fn handle_request(
    request: Request<Incoming>,
    pool: &sqlx::MySqlPool,
    request_id: u64,
) -> Result<Response<Full<Bytes>>, Infallible> {
    if request.method() == hyper::Method::GET && request.uri().path() == "/health" {
        return Ok(Response::new(Full::new(Bytes::from_static(b"ok"))));
    }

    if request.method() == hyper::Method::GET && request.uri().path() == "/clash/villages" {
        return Ok(clash::controller::list_villages(pool, request_id).await);
    }

    if request.method() == hyper::Method::POST && request.uri().path() == "/clash/import" {
      return Ok(clash::controller::import_village(request, pool, request_id).await);
    }

    let mut response = Response::new(Full::new(Bytes::from_static(b"not found")));
    *response.status_mut() = hyper::StatusCode::NOT_FOUND;

    Ok(response)
}

/// Runs a request and writes one line about it: method, path with query, status, client, user agent,
/// referer, sizes and duration. The level follows the status: info below 400, warn for 4xx, error for 5xx.
async fn handle_and_log(
    request: Request<Incoming>,
    pool: &sqlx::MySqlPool,
    client: std::net::SocketAddr,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let request_id = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed);
    let started = std::time::Instant::now();
    let method = request.method().to_string();
    let url = request
        .uri()
        .path_and_query()
        .map(|path| path.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());
    let header = |name: &str| {
        request
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.chars().take(500).collect::<String>())
    };
    let user_agent = header("user-agent");
    let referrer = header("referer");
    let forwarded_for = header("x-forwarded-for");
    let host = header("host");
    let bytes_in = header("content-length").and_then(|value| value.parse::<u64>().ok());
    let version = format!("{:?}", request.version());

    let response = handle_request(request, pool, request_id).await?;
    let status = response.status().as_u16();

    let fields = json!({
        "request_id": request_id,
        "method": method,
        "url": url.chars().take(2000).collect::<String>(),
        "http_version": version,
        "status": status,
        "remote_addr": client.ip().to_string(),
        "remote_port": client.port(),
        "host": host,
        "user_agent": user_agent,
        "referrer": referrer,
        "x_forwarded_for": forwarded_for,
        "bytes_in": bytes_in,
        "bytes_out": response.body().size_hint().exact(),
        "duration_ms": started.elapsed().as_secs_f64() * 1000.0,
    });
    match status {
        500.. => log::error("request", fields),
        400.. => log::warn("request", fields),
        _ => log::info("request", fields),
    }

    Ok(response)
}

/// Logs why the server cannot start, then exits with an error, so systemd restarts it and the reason is in
/// the log instead of only in a panic message.
fn fail(msg: &str, error: impl std::fmt::Display) -> ! {
    log::error(msg, json!({ "error": error.to_string() }));
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    log::info("server starting", json!({ "version": env!("CARGO_PKG_VERSION"), "pid": std::process::id() }));

    if let Err(error) = dotenvy::dotenv()
        && !error.not_found()
    {
        fail("could not load .env", error);
    }

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| fail("DATABASE_URL is not set", "set DATABASE_URL before starting the server"));

    let started = std::time::Instant::now();
    let pool = sqlx::MySqlPool::connect(&database_url)
        .await
        .unwrap_or_else(|error| fail("could not connect to MariaDB", error));
    log::info("connected to MariaDB", json!({ "duration_ms": started.elapsed().as_millis() as u64 }));

    // BIND_ADDR moves the server, for example to run a second copy for a test. Default: 127.0.0.1:4444.
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:4444".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|error| fail(&format!("could not bind {bind_addr}"), error));
    log::info("listening", json!({ "addr": bind_addr }));

    loop {
        let (stream, client_addr) = match listener.accept().await {
            Ok(accepted) => accepted,
            Err(error) => {
                // A full file table or a client that is gone: log it and keep serving.
                // A short pause, so an error that persists cannot spin and flood the log.
                log::warn("could not accept a connection", json!({ "error": error.to_string() }));
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                continue;
            }
        };

        log::info(
            "client connected",
            json!({ "remote_addr": client_addr.ip().to_string(), "remote_port": client_addr.port() }),
        );

        let pool = pool.clone();
        tokio::spawn(async move {
            if let Err(error) = hyper::server::conn::http1::Builder::new()
                .serve_connection(
                    hyper_util::rt::TokioIo::new(stream),
                    hyper::service::service_fn(move |request| {
                        let pool = pool.clone();
                        async move { handle_and_log(request, &pool, client_addr).await }
                    }),
                )
                .await
            {
                log::warn(
                    "connection ended with an error",
                    json!({
                        "remote_addr": client_addr.ip().to_string(),
                        "remote_port": client_addr.port(),
                        "error": error.to_string(),
                    }),
                );
            }
        });
    }
}

