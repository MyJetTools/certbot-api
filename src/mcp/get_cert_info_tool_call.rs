use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetCertInfoInputData {
    #[property(description = "Domain name to inspect")]
    pub domain: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetCertInfoResponse {
    #[property(description = "Common Name from the certificate subject")]
    pub cn: String,

    #[property(description = "Certificate expiration timestamp in RFC-3339")]
    pub expires: String,
}

pub struct GetCertInfoHandler {
    _app: Arc<AppContext>,
}

impl GetCertInfoHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

impl ToolDefinition for GetCertInfoHandler {
    const FUNC_NAME: &'static str = "get_cert_info";
    const DESCRIPTION: &'static str =
        "Return the CN and expiration date for the certificate currently issued for the domain.";
}

#[async_trait::async_trait]
impl McpToolCall<GetCertInfoInputData, GetCertInfoResponse> for GetCertInfoHandler {
    async fn execute_tool_call(
        &self,
        model: GetCertInfoInputData,
    ) -> Result<GetCertInfoResponse, String> {
        let cert_info = crate::scripts::get_cert_info(model.domain.as_str()).await?;
        Ok(GetCertInfoResponse {
            cn: cert_info.cn,
            expires: cert_info.expires.to_rfc3339(),
        })
    }
}
