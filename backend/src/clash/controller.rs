use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::{CONTENT_TYPE, HeaderValue};
use hyper::{Response, StatusCode};


pub fn list_villages() -> Response<Full<Bytes>> {
  let villages = super::service::list_villages();

  match serde_json::to_vec(&villages) {
    Ok(json) => {
      let mut response = Response::new(Full::new(Bytes::from(json)));
      response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

      response
    }
    Err(error) => {
      eprintln!("Could not encode villages: {error}");
      let mut response = Response::new(Full::new(Bytes::from_static(b"Could not encode village")));

      *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
      response
    }
  }
}