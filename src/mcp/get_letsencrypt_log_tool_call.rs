use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetLetsEncryptLogInputData {
    #[property(
        description = "Return only the last N lines of the log. Omit to get the whole file (can be large — prefer a tail of 100-300 lines)."
    )]
    pub tail_lines: Option<u32>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetLetsEncryptLogResponse {
    #[property(description = "Contents of /var/log/letsencrypt/letsencrypt.log")]
    pub log: String,
}

pub struct GetLetsEncryptLogHandler {
    _app: Arc<AppContext>,
}

impl GetLetsEncryptLogHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

impl ToolDefinition for GetLetsEncryptLogHandler {
    const FUNC_NAME: &'static str = "get_letsencrypt_log";
    const DESCRIPTION: &'static str =
        "Read /var/log/letsencrypt/letsencrypt.log — the detailed certbot log. Useful for diagnosing failed renewals: certbot's stderr often just says 'see the logfile'. Pass tail_lines to limit output to the last N lines.";
}

#[async_trait::async_trait]
impl McpToolCall<GetLetsEncryptLogInputData, GetLetsEncryptLogResponse>
    for GetLetsEncryptLogHandler
{
    async fn execute_tool_call(
        &self,
        model: GetLetsEncryptLogInputData,
    ) -> Result<GetLetsEncryptLogResponse, String> {
        let log =
            crate::scripts::get_letsencrypt_log(model.tail_lines.map(|n| n as usize)).await?;

        Ok(GetLetsEncryptLogResponse { log })
    }
}
