use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct AddDomainInputData {
    #[property(
        description = "Domain name to issue a certificate for. Bare domain (example.com) issues a cert with both apex and *.example.com SANs; '*.example.com' is also accepted."
    )]
    pub domain: String,

    #[property(description = "Email address used for ACME registration with Let's Encrypt")]
    pub email: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct AddDomainResponse {
    #[property(description = "Stdout output from certbot on successful issuance")]
    pub output: String,
}

pub struct AddDomainHandler {
    _app: Arc<AppContext>,
}

impl AddDomainHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

impl ToolDefinition for AddDomainHandler {
    const FUNC_NAME: &'static str = "add_domain";
    const DESCRIPTION: &'static str =
        "Issue a new Let's Encrypt certificate for a domain via certbot using Cloudflare DNS-01 challenge. Issues a single cert covering apex and wildcard SANs.";
}

#[async_trait::async_trait]
impl McpToolCall<AddDomainInputData, AddDomainResponse> for AddDomainHandler {
    async fn execute_tool_call(
        &self,
        model: AddDomainInputData,
    ) -> Result<AddDomainResponse, String> {
        let output = crate::scripts::add_domain(model.domain, model.email).await?;
        Ok(AddDomainResponse { output })
    }
}
