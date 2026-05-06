use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::{AppContext, CertJobStatus};

#[http_route(
    method: "GET",
    route: "/api/certbot/v1/check-renew",
    summary: "Check Renewal Status",
    description: "Return the status of a renewal job started via /api/certbot/v1/start-renew. Status is one of 'running', 'completed', 'failed', 'not_found'.",
    controller: "CertBot",
    input_data: "CheckRenewInputModel",

    result:[
        {status_code: 200, description: "Renewal job status", model: CheckRenewHttpModel},
    ]
)]
pub struct CheckRenewAction {
    app: Arc<AppContext>,
}

impl CheckRenewAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &CheckRenewAction,
    input_data: CheckRenewInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let response = match action.app.get_renew_job_status(&input_data.domain).await {
        None => CheckRenewHttpModel {
            status: "not_found".to_string(),
            output: None,
            error: None,
        },
        Some(CertJobStatus::Running) => CheckRenewHttpModel {
            status: "running".to_string(),
            output: None,
            error: None,
        },
        Some(CertJobStatus::Completed { output }) => CheckRenewHttpModel {
            status: "completed".to_string(),
            output: Some(output),
            error: None,
        },
        Some(CertJobStatus::Failed { error }) => CheckRenewHttpModel {
            status: "failed".to_string(),
            output: None,
            error: Some(error),
        },
    };

    HttpOutput::as_json(response).into_ok_result(true).into()
}

#[derive(MyHttpInput)]
pub struct CheckRenewInputModel {
    #[http_query(name = "domain", description = "Domain whose renewal job status should be returned")]
    pub domain: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct CheckRenewHttpModel {
    pub status: String,
    pub output: Option<String>,
    pub error: Option<String>,
}
