use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::{start_renew_job, AppContext, StartRenewOutcome};

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct RenewCertificateInputData {
    #[property(description = "Domain name whose certificate should be renewed")]
    pub domain: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct RenewCertificateResponse {
    #[property(
        description = "Job state right after the call: 'started' if a renewal task was spawned, 'already_running' if a previous task is still in flight"
    )]
    pub status: String,

    #[property(
        description = "Human-readable note. Poll get_renew_status with the same domain to learn when the job finishes."
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
        "Start an asynchronous renewal of an existing Let's Encrypt certificate. Returns immediately so the MCP client never times out — poll get_renew_status with the same domain to read the result.";
}

#[async_trait::async_trait]
impl McpToolCall<RenewCertificateInputData, RenewCertificateResponse> for RenewCertificateHandler {
    async fn execute_tool_call(
        &self,
        model: RenewCertificateInputData,
    ) -> Result<RenewCertificateResponse, String> {
        let outcome = start_renew_job(self.app.clone(), model.domain.clone()).await;

        let response = match outcome {
            StartRenewOutcome::Started => RenewCertificateResponse {
                status: "started".to_string(),
                message: format!(
                    "Renewal for '{}' started. Poll get_renew_status to read the result.",
                    model.domain
                ),
            },
            StartRenewOutcome::AlreadyRunning => RenewCertificateResponse {
                status: "already_running".to_string(),
                message: format!(
                    "Renewal for '{}' is already in progress. Poll get_renew_status to read the result.",
                    model.domain
                ),
            },
        };

        Ok(response)
    }
}
