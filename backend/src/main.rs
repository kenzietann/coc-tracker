mod clash;

use std::convert::Infallible;

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};

async fn handle_request(
    request: Request<Incoming>,
    pool: &sqlx::MySqlPool,
) -> Result<Response<Full<Bytes>>, Infallible> {
    if request.method() == hyper::Method::GET && request.uri().path() == "/health" {
        return Ok(Response::new(Full::new(Bytes::from_static(b"ok"))));
    }

    if request.method() == hyper::Method::GET && request.uri().path() == "/clash/villages" {
        return Ok(clash::controller::list_villages(pool).await);
    }

    if request.method() == hyper::Method::POST && request.uri().path() == "/clash/import" {
      return Ok(clash::controller::import_village(request, pool).await);
    }

    let mut response = Response::new(Full::new(Bytes::from_static(b"not found")));
    *response.status_mut() = hyper::StatusCode::NOT_FOUND;

    Ok(response)
}

/// Runs a request and writes one JSON line about it to standard output: time, method, path, status,
/// client and user agent, so log readers and detection tools can see every request. serde_json escapes
/// every value, so text a client sends can never start a new log line.
async fn handle_and_log(
    request: Request<Incoming>,
    pool: &sqlx::MySqlPool,
    client: std::net::SocketAddr,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let started = std::time::Instant::now();
    let time_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0);
    let method = request.method().to_string();
    let url = request
        .uri()
        .path_and_query()
        .map(|path| path.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());
    let header = |name: hyper::header::HeaderName| {
        request
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.chars().take(500).collect::<String>())
    };
    let user_agent = header(hyper::header::USER_AGENT);
    let referrer = header(hyper::header::REFERER);

    let response = handle_request(request, pool).await?;

    let line = serde_json::json!({
        "time": time_ms,
        "level": "info",
        "msg": "request",
        "method": method,
        "url": url.chars().take(2000).collect::<String>(),
        "status": response.status().as_u16(),
        "remote_addr": client.ip().to_string(),
        "user_agent": user_agent,
        "referrer": referrer,
        "duration_ms": started.elapsed().as_millis() as u64,
    });
    println!("{line}");

    Ok(response)
}

#[tokio::main]
async fn main() {
    println!("CoC tracker backend starting");

    if let Err(error) = dotenvy::dotenv()
        && !error.not_found()
    {
        panic!("Could not load .env");
    }

    let database_url =
        std::env::var("DATABASE_URL").expect("set DATABASE_URL before starting the server");

    let pool = sqlx::MySqlPool::connect(&database_url)
        .await
        .expect("could not connect to MariaDB");

    // BIND_ADDR moves the server, for example to run a second copy for a test. Default: 127.0.0.1:4444.
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:4444".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|error| panic!("could not bind {bind_addr}: {error}"));

    loop {
        let (stream, client_addr) = listener
            .accept()
            .await
            .expect("could not accept connection");

        println!("Client connected from {client_addr}");

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
                eprintln!("Could not serve connection: {error}");
            }
        });
    }
}
