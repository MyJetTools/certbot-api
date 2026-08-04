use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::{AppContext, TaskStatus};

#[http_route(
    method: "GET",
    route: "/api/http01/v1/status",
    summary: "Check HTTP-01 Task Status",
    description: "Return the status of the most recent task for a certificate (typically started via /api/http01/v1/init or /api/http01/v1/reissue). Status is one of 'running', 'completed', 'failed', 'not_found'. Query by the certificate name (the first / primary domain the task was keyed under).",
    controller: "Http01",
    input_data: "Http01StatusInputModel",

    result:[
        {status_code: 200, description: "Task status", model: Http01StatusHttpModel},
    ]
)]
pub struct Http01StatusAction {
    app: Arc<AppContext>,
}

impl Http01StatusAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &Http01StatusAction,
    input_data: Http01StatusInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let response = match action.app.get_task_for_domain(&input_data.domain).await {
        None => Http01StatusHttpModel {
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
            Http01StatusHttpModel {
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
pub struct Http01StatusInputModel {
    #[http_query(
        name = "domain",
        description = "Certificate name (the primary domain the task was keyed under) whose task status should be returned"
    )]
    pub domain: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct Http01StatusHttpModel {
    pub status: String,
    pub task_id: Option<String>,
    pub output: Option<String>,
    pub error: Option<String>,
}
