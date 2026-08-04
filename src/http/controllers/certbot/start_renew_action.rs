use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::{start_renew_dns_task, AppContext, StartTaskOutcome};
use crate::http::errors::prepare_error_to_fail;

#[http_route(
    method: "POST",
    route: "/api/certbot/v1/start-renew",
    summary: "Start Certificate Renewal",
    description: "Spawn a background certbot DNS-01 renewal task and return its task id immediately. The renewed certificate always covers both the apex domain and its wildcard (example.com + *.example.com) in one cert, even if the old one was missing one of them. If a task for this certificate is already running, its id is returned instead of starting a second one. Refuses a certificate that was issued via HTTP-01 (webroot) — use /api/http01/v1/reissue for those. Poll /api/certbot/v1/check-renew (or /api/tasks/v1/list) to read the result.",
    controller: "CertBot",
    input_data: "StartRenewInputModel",

    result:[
        {status_code: 200, description: "Task state right after the call", model: StartRenewHttpModel},
        {status_code: 400, description: "Invalid request (e.g. the certificate was issued via HTTP-01)"},
        {status_code: 500, description: "Server-side failure reading the renewal config"},
    ]
)]
pub struct StartRenewAction {
    app: Arc<AppContext>,
}

impl StartRenewAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &StartRenewAction,
    input_data: StartRenewInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let started = start_renew_dns_task(action.app.clone(), input_data.domain)
        .await
        .map_err(prepare_error_to_fail)?;

    let cert_name = &started.task.cert_name;
    let (status, message) = match started.outcome {
        StartTaskOutcome::Started => (
            "started",
            format!(
                "Renewal for '{}' started. Poll /api/certbot/v1/check-renew to read the result.",
                cert_name
            ),
        ),
        StartTaskOutcome::AlreadyRunning => (
            "already_running",
            format!(
                "Renewal for '{}' is already in progress. Poll /api/certbot/v1/check-renew to read the result.",
                cert_name
            ),
        ),
    };

    let response = StartRenewHttpModel {
        status: status.to_string(),
        task_id: started.task.id.clone(),
        message,
    };

    HttpOutput::as_json(response).into_ok_result(true).into()
}

#[derive(MyHttpInput)]
pub struct StartRenewInputModel {
    #[http_body(name = "domain", description = "Domain name whose certificate should be renewed")]
    pub domain: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct StartRenewHttpModel {
    pub status: String,
    pub task_id: String,
    pub message: String,
}
