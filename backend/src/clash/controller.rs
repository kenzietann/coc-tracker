use hyper::body::Bytes;
use hyper::Request;
use hyper::body::Incoming;
use http_body_util::{BodyExt, Full, Limited};
use hyper::header::{CONTENT_TYPE, HeaderValue};
use hyper::{Response, StatusCode};

pub async fn list_villages(pool: &sqlx::MySqlPool) -> Response<Full<Bytes>> {
    let villages = match super::service::list_villages(pool).await {
        Ok(villages) => villages,
        Err(error) => {
            eprintln!("Could not read villages: {error}");
            let mut response =
                Response::new(Full::new(Bytes::from_static(b"Could not read villages")));
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return response;
        }
    };

    let response_villages: Vec<_> = villages
        .into_iter()
        .map(|(id, tag, label, export_timestamp, raw_export)| {
            let mut timed_entries = Vec::new();

            match serde_json::from_str::<serde_json::Value>(&raw_export) {
                Ok(export) => super::service::collect_timed_entries(&export, &mut timed_entries),
                Err(error) => eprintln!("Could not parse export for {tag}: {error}"),
            }
            serde_json::json!({
              "id": id,
              "tag": tag,
              "label": label,
              "exportTimestamp": export_timestamp,
              "timedEntries": timed_entries,
            })
        })
        .collect();

    match serde_json::to_vec(&response_villages) {
        Ok(json) => {
            let mut response = Response::new(Full::new(Bytes::from(json)));
            response
                .headers_mut()
                .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            response
        }
        Err(error) => {
            eprintln!("Could not encode villages: {error}");
            let mut response =
                Response::new(Full::new(Bytes::from_static(b"Could not encode village")));

            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            response
        }
    }
}

pub async fn import_village(request: Request<Incoming>, pool: &sqlx::MySqlPool) -> Response<Full<Bytes>> {
  let mut response = match Limited::new(request.into_body(), 1024 * 1024)
    .collect()
    .await
  {
    Ok(body) => match serde_json::from_slice::<serde_json::Value>(&body.to_bytes()) {
      Ok(export) => {
        let tag = export["tag"]
          .as_str()
          .map(str::trim)
          .filter(|tag| !tag.is_empty() && tag.len() <= 32);
        let timestamp = export["timestamp"].as_i64().filter(|&value| value >= 0);

        if let (Some(tag), Some(timestamp)) = (tag, timestamp) {
          let raw_export = export.to_string();

          let (status, message) =
            match super::service::save_village(pool, tag, timestamp, &raw_export).await {
              Ok(true) => (StatusCode::OK, "ok"),
              Ok(false) => (StatusCode::CONFLICT, "A newer export has already been imported"),
              Err(error) => {
                eprintln!("Could not save village: {error}");
              (StatusCode::INTERNAL_SERVER_ERROR, "Could not save village")
              }
            };

          let mut response = Response::new(Full::new(Bytes::from_static(message.as_bytes())));
          *response.status_mut() = status;
          return response;
        }

        Response::new(Full::new(Bytes::from_static(b"Missing or invalid tag or timestamp")))
      }
      Err(_) => Response::new(Full::new(Bytes::from_static(b"Invalid JSON"))),
    },
    Err(_) => Response::new(Full::new(Bytes::from_static(b"Could not read request body"))),
  };

  *response.status_mut() = StatusCode::BAD_REQUEST;
  response
}