use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetFullchainInputData {
    #[property(description = "Domain name whose fullchain certificate should be returned")]
    pub domain: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetFullchainResponse {
    #[property(description = "PEM-encoded fullchain (cert + intermediate) contents")]
    pub fullchain: String,
}

pub struct GetFullchainHandler {
    _app: Arc<AppContext>,
}

impl GetFullchainHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

impl ToolDefinition for GetFullchainHandler {
    const FUNC_NAME: &'static str = "get_fullchain";
    const DESCRIPTION: &'static str =
        "Read the fullchain.pem file (leaf certificate + chain) for the specified domain.";
}

#[async_trait::async_trait]
impl McpToolCall<GetFullchainInputData, GetFullchainResponse> for GetFullchainHandler {
    async fn execute_tool_call(
        &self,
        model: GetFullchainInputData,
    ) -> Result<GetFullchainResponse, String> {
        let fullchain = crate::scripts::get_fullchain(model.domain.as_str()).await?;
        Ok(GetFullchainResponse { fullchain })
    }
}
