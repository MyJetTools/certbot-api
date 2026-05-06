use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetPrivateKeyInputData {
    #[property(description = "Domain name whose private key should be returned")]
    pub domain: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetPrivateKeyResponse {
    #[property(description = "PEM-encoded private key contents")]
    pub private_key: String,
}

pub struct GetPrivateKeyHandler {
    _app: Arc<AppContext>,
}

impl GetPrivateKeyHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

impl ToolDefinition for GetPrivateKeyHandler {
    const FUNC_NAME: &'static str = "get_private_key";
    const DESCRIPTION: &'static str =
        "Read the private key file (privkey.pem) for the specified domain certificate.";
}

#[async_trait::async_trait]
impl McpToolCall<GetPrivateKeyInputData, GetPrivateKeyResponse> for GetPrivateKeyHandler {
    async fn execute_tool_call(
        &self,
        model: GetPrivateKeyInputData,
    ) -> Result<GetPrivateKeyResponse, String> {
        let private_key = crate::scripts::get_private_key(model.domain.as_str()).await?;
        Ok(GetPrivateKeyResponse { private_key })
    }
}
