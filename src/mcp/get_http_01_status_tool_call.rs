use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::{AppContext, TaskStatus};

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetHttp01StatusInputData {
    #[property(
        description = "Certificate name (the primary domain the task was keyed under) whose task status should be returned"
    )]
    pub domain: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetHttp01StatusResponse {
    #[property(
        description = "One of: 'running', 'completed', 'failed', 'not_found' (no task has been started for this certificate since the service booted)"
    )]
    pub status: String,

    #[property(description = "Id of the matched task, when one exists.")]
    pub task_id: Option<String>,

    #[property(description = "Stdout output from certbot — present only when status == 'completed'")]
    pub output: Option<String>,

    #[property(description = "Error message — present only when status == 'failed'")]
    pub error: Option<String>,
}

pub struct GetHttp01StatusHandler {
    app: Arc<AppContext>,
}

impl GetHttp01StatusHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for GetHttp01StatusHandler {
    const FUNC_NAME: &'static str = "get_http_01_status";
    const DESCRIPTION: &'static str =
        "Return the status of the most recent task for a certificate, typically started via init_http_01 or reissue_http_01. Use this to poll until the task finishes instead of waiting on a single long-running call.";
}

#[async_trait::async_trait]
impl McpToolCall<GetHttp01StatusInputData, GetHttp01StatusResponse> for GetHttp01StatusHandler {
    async fn execute_tool_call(
        &self,
        model: GetHttp01StatusInputData,
    ) -> Result<GetHttp01StatusResponse, String> {
        let response = match self.app.get_task_for_domain(&model.domain).await {
            None => GetHttp01StatusResponse {
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
                GetHttp01StatusResponse {
                    status: task.status.as_str().to_string(),
                    task_id: Some(task.id.clone()),
                    output,
                    error,
                }
            }
        };

        Ok(response)
    }
}
