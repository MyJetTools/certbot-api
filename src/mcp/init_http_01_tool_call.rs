use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::{start_init_http_01_task, AppContext, StartTaskOutcome, StartedTask};

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct InitHttp01InputData {
    #[property(
        description = "Exact hostnames to include in the certificate (SANs). HTTP-01 cannot issue wildcards. The first hostname names the certificate lineage."
    )]
    pub domains: Vec<String>,

    #[property(description = "Email address used for ACME registration with Let's Encrypt")]
    pub email: String,

    #[property(
        description = "Optional webroot directory certbot writes ACME challenge files into. Defaults to /var/www/acme when omitted."
    )]
    pub webroot: Option<String>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct StartTaskResponse {
    #[property(
        description = "Task state right after the call: 'started' if a task was spawned, 'already_running' if a task for this certificate was already in flight"
    )]
    pub status: String,

    #[property(description = "Id of the task — pass it to list_tasks / poll get_http_01_status to track it.")]
    pub task_id: String,

    #[property(description = "Certificate name (primary domain) the task is keyed under.")]
    pub cert_name: String,

    #[property(description = "Human-readable note describing how to poll for the result.")]
    pub message: String,
}

impl StartTaskResponse {
    pub fn from_started(started: &StartedTask, poll_hint: &str) -> Self {
        let cert_name = started.task.cert_name.clone();
        let (status, message) = match started.outcome {
            StartTaskOutcome::Started => (
                "started",
                format!("Task for '{}' started. {}", cert_name, poll_hint),
            ),
            StartTaskOutcome::AlreadyRunning => (
                "already_running",
                format!(
                    "A certificate task for '{}' is already in progress. {}",
                    cert_name, poll_hint
                ),
            ),
        };

        StartTaskResponse {
            status: status.to_string(),
            task_id: started.task.id.clone(),
            cert_name,
            message: message.to_string(),
        }
    }
}

pub struct InitHttp01Handler {
    app: Arc<AppContext>,
}

impl InitHttp01Handler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for InitHttp01Handler {
    const FUNC_NAME: &'static str = "init_http_01";
    const DESCRIPTION: &'static str =
        "Start an asynchronous task that issues a new Let's Encrypt certificate using the HTTP-01 challenge (certbot webroot plugin). Certbot writes the challenge token under <webroot>/.well-known/acme-challenge/, which the component owning port 80 must serve publicly. Validates each exact FQDN over :80 and cannot issue wildcards — list every hostname. Returns a task_id immediately (or the id of an already-running task) so the MCP client never times out — poll get_http_01_status with the returned cert_name, or list_tasks, to read the result.";
}

#[async_trait::async_trait]
impl McpToolCall<InitHttp01InputData, StartTaskResponse> for InitHttp01Handler {
    async fn execute_tool_call(
        &self,
        model: InitHttp01InputData,
    ) -> Result<StartTaskResponse, String> {
        let started =
            start_init_http_01_task(self.app.clone(), model.domains, model.email, model.webroot)
                .await
                .map_err(|e| e.to_string())?;

        Ok(StartTaskResponse::from_started(
            &started,
            "Poll get_http_01_status with this cert_name to read the result.",
        ))
    }
}
