use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::{start_renew_dns_task, AppContext, StartTaskOutcome};

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct RenewCertificateInputData {
    #[property(description = "Domain name whose certificate should be renewed")]
    pub domain: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct RenewCertificateResponse {
    #[property(
        description = "Task state right after the call: 'started' if a task was spawned, 'already_running' if a previous task is still in flight"
    )]
    pub status: String,

    #[property(description = "Id of the task — pass it to list_tasks / poll get_renew_status to track it.")]
    pub task_id: String,

    #[property(
        description = "Human-readable note. Poll get_renew_status with the same domain to learn when the task finishes."
    )]
    pub message: String,
}

pub struct RenewCertificateHandler {
    app: Arc<AppContext>,
}

impl RenewCertificateHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for RenewCertificateHandler {
    const FUNC_NAME: &'static str = "renew_certificate";
    const DESCRIPTION: &'static str =
        "Start an asynchronous DNS-01 renewal of an existing Let's Encrypt certificate. The renewed certificate always covers both the apex domain and its wildcard (example.com + *.example.com) in one cert, even if the old one was missing one of them. Refuses a certificate that was issued via HTTP-01 (webroot) — use reissue_http_01 for those. Returns a task_id immediately so the MCP client never times out — poll get_renew_status with the same domain, or list_tasks, to read the result.";
}

#[async_trait::async_trait]
impl McpToolCall<RenewCertificateInputData, RenewCertificateResponse> for RenewCertificateHandler {
    async fn execute_tool_call(
        &self,
        model: RenewCertificateInputData,
    ) -> Result<RenewCertificateResponse, String> {
        let started = start_renew_dns_task(self.app.clone(), model.domain)
            .await
            .map_err(|e| e.to_string())?;

        let cert_name = started.task.cert_name.clone();
        let (status, message) = match started.outcome {
            StartTaskOutcome::Started => (
                "started",
                format!(
                    "Renewal for '{}' started. Poll get_renew_status to read the result.",
                    cert_name
                ),
            ),
            StartTaskOutcome::AlreadyRunning => (
                "already_running",
                format!(
                    "Renewal for '{}' is already in progress. Poll get_renew_status to read the result.",
                    cert_name
                ),
            ),
        };

        Ok(RenewCertificateResponse {
            status: status.to_string(),
            task_id: started.task.id.clone(),
            message,
        })
    }
}
