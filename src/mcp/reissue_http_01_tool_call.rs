use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::{start_reissue_http_01_task, AppContext};

use super::StartTaskResponse;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct ReissueHttp01InputData {
    #[property(description = "Primary domain naming the HTTP-01 certificate lineage to reissue")]
    pub domain: String,
}

pub struct ReissueHttp01Handler {
    app: Arc<AppContext>,
}

impl ReissueHttp01Handler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for ReissueHttp01Handler {
    const FUNC_NAME: &'static str = "reissue_http_01";
    const DESCRIPTION: &'static str =
        "Start an asynchronous task that force-reissues an existing HTTP-01 certificate now, restoring its stored domain set and webroot path from certbot's renewal config. Refuses a certificate issued with a non-webroot authenticator (e.g. a DNS-01 wildcard) — use renew_certificate for those. Returns a task_id immediately — poll get_http_01_status with the returned cert_name, or list_tasks, to read the result.";
}

#[async_trait::async_trait]
impl McpToolCall<ReissueHttp01InputData, StartTaskResponse> for ReissueHttp01Handler {
    async fn execute_tool_call(
        &self,
        model: ReissueHttp01InputData,
    ) -> Result<StartTaskResponse, String> {
        let started = start_reissue_http_01_task(self.app.clone(), model.domain)
            .await
            .map_err(|e| e.to_string())?;

        Ok(StartTaskResponse::from_started(
            &started,
            "Poll get_http_01_status with this cert_name to read the result.",
        ))
    }
}
