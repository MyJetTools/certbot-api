use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::{AppContext, TaskStatus};

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct ListTasksInputData {}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct TaskModel {
    #[property(description = "Task id")]
    pub id: String,

    #[property(description = "Certificate name (primary domain) the task is keyed under")]
    pub cert_name: String,

    #[property(description = "Full domain (SAN) set the task targets")]
    pub domains: Vec<String>,

    #[property(description = "Operation kind: 'issue-http-01', 'reissue-http-01' or 'renew-dns'")]
    pub kind: String,

    #[property(description = "'running', 'completed' or 'failed'")]
    pub status: String,

    #[property(description = "Stdout output from certbot — present only when status == 'completed'")]
    pub output: Option<String>,

    #[property(description = "Error message — present only when status == 'failed'")]
    pub error: Option<String>,

    #[property(description = "RFC-3339 timestamp the task was created")]
    pub created_at: String,

    #[property(description = "RFC-3339 timestamp the task last changed state")]
    pub updated_at: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct ListTasksResponse {
    #[property(description = "Every certificate task tracked since boot, newest first")]
    pub tasks: Vec<TaskModel>,
}

pub struct ListTasksHandler {
    app: Arc<AppContext>,
}

impl ListTasksHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for ListTasksHandler {
    const FUNC_NAME: &'static str = "list_tasks";
    const DESCRIPTION: &'static str =
        "List every certificate task (issue / reissue / renew) tracked since the service started, newest first — the ones running right now and the outcome of recent ones.";
}

#[async_trait::async_trait]
impl McpToolCall<ListTasksInputData, ListTasksResponse> for ListTasksHandler {
    async fn execute_tool_call(
        &self,
        _model: ListTasksInputData,
    ) -> Result<ListTasksResponse, String> {
        let tasks = self
            .app
            .list_tasks()
            .await
            .iter()
            .map(|task| {
                let (output, error) = match &task.status {
                    TaskStatus::Running => (None, None),
                    TaskStatus::Completed { output } => (Some(output.clone()), None),
                    TaskStatus::Failed { error } => (None, Some(error.clone())),
                };
                TaskModel {
                    id: task.id.clone(),
                    cert_name: task.cert_name.clone(),
                    domains: task.domains.clone(),
                    kind: task.kind.as_str().to_string(),
                    status: task.status.as_str().to_string(),
                    output,
                    error,
                    created_at: task.created_at.to_rfc3339(),
                    updated_at: task.updated_at.to_rfc3339(),
                }
            })
            .collect();

        Ok(ListTasksResponse { tasks })
    }
}
