use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct UpdateCloudflareConfigInputData {
    #[property(description = "Cloudflare API token to be written to /cloudflare.ini")]
    pub api_token: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct UpdateCloudflareConfigResponse {
    #[property(description = "True if the config file was updated")]
    pub success: bool,
}

pub struct UpdateCloudflareConfigHandler {
    _app: Arc<AppContext>,
}

impl UpdateCloudflareConfigHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

impl ToolDefinition for UpdateCloudflareConfigHandler {
    const FUNC_NAME: &'static str = "update_cloudflare_config";
    const DESCRIPTION: &'static str =
        "Write the provided Cloudflare API token to /cloudflare.ini so certbot can perform DNS-01 challenges.";
}

#[async_trait::async_trait]
impl McpToolCall<UpdateCloudflareConfigInputData, UpdateCloudflareConfigResponse>
    for UpdateCloudflareConfigHandler
{
    async fn execute_tool_call(
        &self,
        model: UpdateCloudflareConfigInputData,
    ) -> Result<UpdateCloudflareConfigResponse, String> {
        crate::scripts::update_cloudflare_config(model.api_token).await;
        Ok(UpdateCloudflareConfigResponse { success: true })
    }
}
