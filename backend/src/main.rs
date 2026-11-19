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

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4444")
        .await
        .expect("could not bind port 4444");

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
                        async move { handle_request(request, &pool).await }
                    }),
                )
                .await
            {
                eprintln!("Could not serve connection: {error}");
            }
        });
    }
}
