use super::REQUEST_ID_HEADER;
use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use tracing::log::warn;

pub async fn set_request(mut req: Request, next: Next) -> Response {
    let id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(v) => Some(v.clone()),
        None => {
            let request_id = uuid::Uuid::now_v7().to_string();
            match HeaderValue::from_str(&request_id) {
                Ok(v) => {
                    req.headers_mut().insert(REQUEST_ID_HEADER, v.clone());
                    Some(v)
                }
                Err(e) => {
                    warn!("parse generated request id failed :{} ", e);
                    None
                }
            }
        }
    };

    let mut res = next.run(req).await;
    let Some(id) = id else {
        return res;
    };
    res.headers_mut().insert(REQUEST_ID_HEADER, id);
    res
}
