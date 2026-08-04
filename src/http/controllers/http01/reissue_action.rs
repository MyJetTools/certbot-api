use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;

use crate::app::{start_reissue_http_01_task, AppContext};
use crate::http::errors::prepare_error_to_fail;

use super::{start_http_01_response, StartTaskHttpModel};

#[http_route(
    method: "POST",
    route: "/api/http01/v1/reissue",
    summary: "Reissue Certificate via HTTP-01",
    description: "Spawn a background task that force-reissues an existing HTTP-01 certificate now, and return its task id immediately. If a task for this certificate is already running, its id is returned instead of starting a second one. Certbot restores the stored domain set and webroot path from its renewal config, so only the certificate name (its first / primary domain) is needed. Refuses a certificate that was issued with a different authenticator (e.g. a DNS-01 wildcard) — use start-renew for those. Poll /api/http01/v1/status (or /api/tasks/v1/list) to read the result.",
    controller: "Http01",
    input_data: "ReissueHttp01InputModel",

    result:[
        {status_code: 200, description: "Task state right after the call", model: StartTaskHttpModel},
        {status_code: 400, description: "Invalid request (wildcard, or the certificate was not issued via HTTP-01)"},
        {status_code: 404, description: "Certificate renewal config not found"},
        {status_code: 500, description: "Server-side failure reading the renewal config"},
    ]
)]
pub struct ReissueHttp01Action {
    app: Arc<AppContext>,
}

impl ReissueHttp01Action {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &ReissueHttp01Action,
    input_data: ReissueHttp01InputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let started = start_reissue_http_01_task(action.app.clone(), input_data.domain)
        .await
        .map_err(prepare_error_to_fail)?;

    let response = start_http_01_response(&started);
    HttpOutput::as_json(response).into_ok_result(true).into()
}

#[derive(MyHttpInput)]
pub struct ReissueHttp01InputModel {
    #[http_body(
        name = "domain",
        description = "Primary domain naming the HTTP-01 certificate lineage to reissue"
    )]
    pub domain: String,
}
