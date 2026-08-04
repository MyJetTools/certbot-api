use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::{AppContext, TaskStatus};

#[http_route(
    method: "GET",
    route: "/api/certbot/v1/check-renew",
    summary: "Check Renewal Status",
    description: "Return the status of the most recent renewal task for a certificate, started via /api/certbot/v1/start-renew. Status is one of 'running', 'completed', 'failed', 'not_found'.",
    controller: "CertBot",
    input_data: "CheckRenewInputModel",

    result:[
        {status_code: 200, description: "Renewal task status", model: CheckRenewHttpModel},
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
    let response = match action.app.get_task_for_domain(&input_data.domain).await {
        None => CheckRenewHttpModel {
            status: "not_found".to_string(),
            task_id: None,
            output: None,
            error: None,
        },
        Some(task) => {
            let (output, error) = match &task.status {
                TaskStatus::Running => (None, None),
                TaskStatus::Completed { output } => (Some(output.clone()), None),
                TaskStatus::Failed { error } => (None, Some(error.clone())),
            };
            CheckRenewHttpModel {
                status: task.status.as_str().to_string(),
                task_id: Some(task.id.clone()),
                output,
                error,
            }
        }
    };

    HttpOutput::as_json(response).into_ok_result(true).into()
}

#[derive(MyHttpInput)]
pub struct CheckRenewInputModel {
    #[http_query(name = "domain", description = "Domain whose renewal task status should be returned")]
    pub domain: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct CheckRenewHttpModel {
    pub status: String,
    pub task_id: Option<String>,
    pub output: Option<String>,
    pub error: Option<String>,
}
