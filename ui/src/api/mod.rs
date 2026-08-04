//! HTTP access to the Certbot API server. This UI is served as static files by
//! that same server, so every call is a root-relative URL resolved against the
//! page origin by FlUrl's wasm fetch backend — there is no base URL to thread
//! through.
//!
//! The models in `crate::models` are plain-serde mirrors of the server's HTTP
//! models (the server crate's `MyHttpObjectStructure` derive is not wasm-clean,
//! so it cannot be shared verbatim). Every optional field is `#[serde(default)]`
//! so an older or newer server still deserializes rather than blanking the page.

use flurl::{FlUrl, FlUrlError, FlUrlResponse};
use serde::de::DeserializeOwned;

use crate::models::{DomainModel, DomainsResponse, RequestError, TaskModel, TasksResponse};

fn is_success(status: u16) -> bool {
    (200..300).contains(&status)
}

async fn read_error_body(response: &mut FlUrlResponse) -> RequestError {
    let status = response.get_status_code();

    let message = response
        .get_body_as_str()
        .await
        .map(|body| body.to_string())
        .unwrap_or_else(|err| format!("{:?}", err));

    RequestError {
        message: format!("HTTP {}: {}", status, message),
    }
}

/// 2xx → deserialize the body into `T`; any other status → `Err` carrying the body.
async fn handle_http_response<T: DeserializeOwned>(
    response: Result<FlUrlResponse, FlUrlError>,
) -> Result<T, RequestError> {
    let mut response = response?;

    if is_success(response.get_status_code()) {
        return Ok(response.get_json().await?);
    }

    Err(read_error_body(&mut response).await)
}

/// Every domain this service manages, with its certificate expiry.
pub async fn get_domains() -> Result<Vec<DomainModel>, RequestError> {
    let response = FlUrl::new("/api/certificates/v1/domains-expiration")
        .get()
        .await;

    let payload: DomainsResponse = handle_http_response(response).await?;

    Ok(payload.domains)
}

/// Every certificate task tracked since the server started, newest first.
pub async fn get_tasks() -> Result<Vec<TaskModel>, RequestError> {
    let response = FlUrl::new("/api/tasks/v1/list").get().await;

    let payload: TasksResponse = handle_http_response(response).await?;

    Ok(payload.tasks)
}
