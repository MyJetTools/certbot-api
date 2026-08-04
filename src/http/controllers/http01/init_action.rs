use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::{start_init_http_01_task, AppContext};
use crate::http::errors::prepare_error_to_fail;

use super::start_http_01_response;

#[http_route(
    method: "POST",
    route: "/api/http01/v1/init",
    summary: "Issue Certificate via HTTP-01",
    description: "Spawn a background task that issues a new Let's Encrypt certificate using the HTTP-01 challenge (certbot webroot plugin) and return its task id immediately. If a task for this certificate is already running, its id is returned instead of starting a second one. Certbot writes the challenge token under <webroot>/.well-known/acme-challenge/, and whatever owns port 80 on this host (e.g. the reverse proxy) must serve that path publicly. HTTP-01 validates each exact FQDN over :80 and CANNOT issue a wildcard — list every hostname explicitly. The certificate lineage is named after the first domain; poll /api/http01/v1/status (or /api/tasks/v1/list) to read the result.",
    controller: "Http01",
    input_data: "InitHttp01InputModel",

    result:[
        {status_code: 200, description: "Task state right after the call", model: StartTaskHttpModel},
        {status_code: 400, description: "Invalid request (empty/wildcard domain, or the name already belongs to a non-webroot certificate)"},
        {status_code: 500, description: "Server-side failure reading the renewal config"},
    ]
)]
pub struct InitHttp01Action {
    app: Arc<AppContext>,
}

impl InitHttp01Action {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &InitHttp01Action,
    input_data: InitHttp01InputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let started = start_init_http_01_task(
        action.app.clone(),
        input_data.domains,
        input_data.email,
        input_data.webroot,
    )
    .await
    .map_err(prepare_error_to_fail)?;

    let response = start_http_01_response(&started);
    HttpOutput::as_json(response).into_ok_result(true).into()
}

#[derive(MyHttpInput)]
pub struct InitHttp01InputModel {
    #[http_body(
        name = "domains",
        description = "Exact hostnames to include in the certificate (SANs). No wildcards. The first one names the lineage."
    )]
    pub domains: Vec<String>,

    #[http_body(
        name = "email",
        description = "Email address for ACME registration with Let's Encrypt"
    )]
    pub email: String,

    #[http_body(
        name = "webroot",
        description = "Optional webroot directory certbot writes challenge files into. Defaults to /var/www/acme."
    )]
    pub webroot: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct StartTaskHttpModel {
    pub status: String,
    pub task_id: String,
    pub cert_name: String,
    pub message: String,
}
