use hyper::body::Bytes;
use hyper::Request;
use hyper::body::Incoming;
use http_body_util::{BodyExt, Full, Limited};
use hyper::header::{CONTENT_TYPE, HeaderValue};
use hyper::{Response, StatusCode};
use serde_json::json;

use crate::log;

pub async fn list_villages(pool: &sqlx::MySqlPool, request_id: u64) -> Response<Full<Bytes>> {
    let villages = match super::service::list_villages(pool).await {
        Ok(villages) => villages,
        Err(error) => {
            log::error("could not read villages", json!({ "request_id": request_id, "error": error.to_string() }));
            let mut response =
                Response::new(Full::new(Bytes::from_static(b"Could not read villages")));
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return response;
        }
    };

    let mut unreadable = 0;
    let response_villages: Vec<_> = villages
        .into_iter()
        .map(|(id, tag, label, export_timestamp, raw_export)| {
            let mut timed_entries = Vec::new();

            match serde_json::from_str::<serde_json::Value>(&raw_export) {
                Ok(export) => super::service::collect_timed_entries(&export, &mut timed_entries),
                Err(error) => {
                    unreadable += 1;
                    log::warn(
                        "could not parse a stored export",
                        json!({ "request_id": request_id, "village_id": id, "tag": tag, "error": error.to_string() }),
                    );
                }
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

    log::info(
        "villages listed",
        json!({ "request_id": request_id, "villages": response_villages.len(), "unreadable_exports": unreadable }),
    );

    match serde_json::to_vec(&response_villages) {
        Ok(json) => {
            let mut response = Response::new(Full::new(Bytes::from(json)));
            response
                .headers_mut()
                .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            response
        }
        Err(error) => {
            log::error("could not encode villages", json!({ "request_id": request_id, "error": error.to_string() }));
            let mut response =
                Response::new(Full::new(Bytes::from_static(b"Could not encode village")));

            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            response
        }
    }
}

/// The largest import body that is read. A larger one is refused before it is parsed.
const MAX_IMPORT_BYTES: usize = 1024 * 1024;

pub async fn import_village(request: Request<Incoming>, pool: &sqlx::MySqlPool, request_id: u64) -> Response<Full<Bytes>> {
  // Every refusal says why in the log, with the request id; the body itself is never logged.
  let refuse = |reason: &str, message: &'static [u8]| {
    log::warn("import refused", json!({ "request_id": request_id, "reason": reason }));
    let mut response = Response::new(Full::new(Bytes::from_static(message)));
    *response.status_mut() = StatusCode::BAD_REQUEST;
    response
  };

  let body = match Limited::new(request.into_body(), MAX_IMPORT_BYTES).collect().await {
    Ok(body) => body.to_bytes(),
    Err(_) => return refuse("body unreadable or larger than 1 MiB", b"Could not read request body"),
  };
  let export = match serde_json::from_slice::<serde_json::Value>(&body) {
    Ok(export) => export,
    Err(_) => return refuse("invalid JSON", b"Invalid JSON"),
  };

  let tag = export["tag"]
    .as_str()
    .map(str::trim)
    .filter(|tag| !tag.is_empty() && tag.len() <= 32);
  let timestamp = export["timestamp"].as_i64().filter(|&value| value >= 0);
  let (Some(tag), Some(timestamp)) = (tag, timestamp) else {
    return refuse("missing or invalid tag or timestamp", b"Missing or invalid tag or timestamp");
  };

  let raw_export = export.to_string();
  let (status, message) = match super::service::save_village(pool, tag, timestamp, &raw_export).await {
    Ok(true) => {
      log::info(
        "village imported",
        json!({ "request_id": request_id, "tag": tag, "export_timestamp": timestamp, "bytes": body.len() }),
      );
      (StatusCode::OK, "ok")
    }
    Ok(false) => {
      log::warn(
        "import refused",
        json!({ "request_id": request_id, "reason": "a newer export is already imported", "tag": tag, "export_timestamp": timestamp }),
      );
      (StatusCode::CONFLICT, "A newer export has already been imported")
    }
    Err(error) => {
      log::error(
        "could not save village",
        json!({ "request_id": request_id, "tag": tag, "export_timestamp": timestamp, "error": error.to_string() }),
      );
      (StatusCode::INTERNAL_SERVER_ERROR, "Could not save village")
    }
  };

  let mut response = Response::new(Full::new(Bytes::from_static(message.as_bytes())));
  *response.status_mut() = status;
  response
}
