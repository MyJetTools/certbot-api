use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetDomainsListInputData {}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetDomainsListResponse {
    #[property(description = "All domains currently managed by certbot on this host")]
    pub domains: Vec<String>,
}

pub struct GetDomainsListHandler {
    _app: Arc<AppContext>,
}

impl GetDomainsListHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

impl ToolDefinition for GetDomainsListHandler {
    const FUNC_NAME: &'static str = "get_domains_list";
    const DESCRIPTION: &'static str =
        "List every domain that currently has a certificate managed by certbot on this host.";
}

#[async_trait::async_trait]
impl McpToolCall<GetDomainsListInputData, GetDomainsListResponse> for GetDomainsListHandler {
    async fn execute_tool_call(
        &self,
        _model: GetDomainsListInputData,
    ) -> Result<GetDomainsListResponse, String> {
        let domains = crate::scripts::get_domains_list().await?;
        Ok(GetDomainsListResponse { domains })
    }
}
