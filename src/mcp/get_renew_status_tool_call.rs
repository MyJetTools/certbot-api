use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::{AppContext, CertJobStatus};

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetRenewStatusInputData {
    #[property(description = "Domain whose renewal job status should be returned")]
    pub domain: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetRenewStatusResponse {
    #[property(
        description = "One of: 'running', 'completed', 'failed', 'not_found' (no job has ever been started for this domain since the server booted)"
    )]
    pub status: String,

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
        "Return the status of a renewal job started via renew_certificate. Use this to poll until the job finishes instead of waiting on a single long-running call.";
}

#[async_trait::async_trait]
impl McpToolCall<GetRenewStatusInputData, GetRenewStatusResponse> for GetRenewStatusHandler {
    async fn execute_tool_call(
        &self,
        model: GetRenewStatusInputData,
    ) -> Result<GetRenewStatusResponse, String> {
        let response = match self.app.get_renew_job_status(&model.domain).await {
            None => GetRenewStatusResponse {
                status: "not_found".to_string(),
                output: None,
                error: None,
            },
            Some(CertJobStatus::Running) => GetRenewStatusResponse {
                status: "running".to_string(),
                output: None,
                error: None,
            },
            Some(CertJobStatus::Completed { output }) => GetRenewStatusResponse {
                status: "completed".to_string(),
                output: Some(output),
                error: None,
            },
            Some(CertJobStatus::Failed { error }) => GetRenewStatusResponse {
                status: "failed".to_string(),
                output: None,
                error: Some(error),
            },
        };
        Ok(response)
    }
}
