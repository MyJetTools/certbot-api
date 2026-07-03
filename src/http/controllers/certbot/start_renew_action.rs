use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::{start_renew_job, AppContext, StartRenewOutcome};

#[http_route(
    method: "POST",
    route: "/api/certbot/v1/start-renew",
    summary: "Start Certificate Renewal",
    description: "Spawn a background certbot renewal task and return immediately. The renewed certificate always covers both the apex domain and its wildcard (example.com + *.example.com) in one cert, even if the old one was missing one of them. Poll /api/certbot/v1/check-renew with the same domain to read the result. Avoids client-side timeouts on slow renewals (DNS-01 propagation can take a minute or more).",
    controller: "CertBot",
    input_data: "StartRenewInputModel",

    result:[
        {status_code: 200, description: "Job state right after the call", model: StartRenewHttpModel},
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
    let outcome = start_renew_job(action.app.clone(), input_data.domain.clone()).await;

    let response = match outcome {
        StartRenewOutcome::Started => StartRenewHttpModel {
            status: "started".to_string(),
            message: format!(
                "Renewal for '{}' started. Poll /api/certbot/v1/check-renew to read the result.",
                input_data.domain
            ),
        },
        StartRenewOutcome::AlreadyRunning => StartRenewHttpModel {
            status: "already_running".to_string(),
            message: format!(
                "Renewal for '{}' is already in progress. Poll /api/certbot/v1/check-renew to read the result.",
                input_data.domain
            ),
        },
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
    pub message: String,
}
