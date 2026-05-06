use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetDomainsWithExpirationInputData {}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct DomainExpirationItem {
    #[property(description = "Domain name")]
    pub domain: String,

    #[property(description = "Certificate expiration timestamp (RFC-3339), if readable")]
    pub expiration: Option<String>,

    #[property(description = "Error message if expiration could not be read for this domain")]
    pub error: Option<String>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetDomainsWithExpirationResponse {
    #[property(description = "All managed domains with their expiration dates or per-domain errors")]
    pub domains: Vec<DomainExpirationItem>,
}

pub struct GetDomainsWithExpirationHandler {
    _app: Arc<AppContext>,
}

impl GetDomainsWithExpirationHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

impl ToolDefinition for GetDomainsWithExpirationHandler {
    const FUNC_NAME: &'static str = "get_domains_with_expiration";
    const DESCRIPTION: &'static str =
        "List every managed domain together with its certificate expiration date — useful for finding certs that need renewal.";
}

#[async_trait::async_trait]
impl McpToolCall<GetDomainsWithExpirationInputData, GetDomainsWithExpirationResponse>
    for GetDomainsWithExpirationHandler
{
    async fn execute_tool_call(
        &self,
        _model: GetDomainsWithExpirationInputData,
    ) -> Result<GetDomainsWithExpirationResponse, String> {
        let items = crate::scripts::get_domains_with_expiration().await?;
        let domains = items
            .into_iter()
            .map(|item| DomainExpirationItem {
                domain: item.domain,
                expiration: item.expiration,
                error: item.error,
            })
            .collect();
        Ok(GetDomainsWithExpirationResponse { domains })
    }
}
