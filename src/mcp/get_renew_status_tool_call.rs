use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::{AppContext, TaskStatus};

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetRenewStatusInputData {
    #[property(description = "Domain whose renewal task status should be returned")]
    pub domain: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetRenewStatusResponse {
    #[property(
        description = "One of: 'running', 'completed', 'failed', 'not_found' (no task has ever been started for this domain since the service booted)"
    )]
    pub status: String,

    #[property(description = "Id of the matched task, when one exists.")]
    pub task_id: Option<String>,

    #[property(description = "Stdout output from certbot — present only when status == 'completed'")]
    pub output: Option<String>,

    #[property(description = "Error message — present only when status == 'failed'")]
    pub error: Option<String>,
}

pub struct GetRenewStatusHandler {
    app: Arc<AppContext>,
}

impl GetRenewStatusHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for GetRenewStatusHandler {
    const FUNC_NAME: &'static str = "get_renew_status";
    const DESCRIPTION: &'static str =
        "Return the status of the most recent renewal task for a certificate, started via renew_certificate. Use this to poll until the task finishes instead of waiting on a single long-running call.";
}

#[async_trait::async_trait]
impl McpToolCall<GetRenewStatusInputData, GetRenewStatusResponse> for GetRenewStatusHandler {
    async fn execute_tool_call(
        &self,
        model: GetRenewStatusInputData,
    ) -> Result<GetRenewStatusResponse, String> {
        let response = match self.app.get_task_for_domain(&model.domain).await {
            None => GetRenewStatusResponse {
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
                GetRenewStatusResponse {
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
