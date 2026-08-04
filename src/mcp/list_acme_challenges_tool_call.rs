use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct ListAcmeChallengesInputData {}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct AcmeChallengeModel {
    #[property(description = "Challenge token — the last segment of the challenge URL / the file name")]
    pub token: String,

    #[property(description = "Key authorization served at that token's URL")]
    pub content: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct ListAcmeChallengesResponse {
    #[property(
        description = "ACME HTTP-01 challenge files currently in the webroot. Usually empty — non-empty only while an init/reissue run is mid-validation."
    )]
    pub challenges: Vec<AcmeChallengeModel>,
}

pub struct ListAcmeChallengesHandler {
    _app: Arc<AppContext>,
}

impl ListAcmeChallengesHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

impl ToolDefinition for ListAcmeChallengesHandler {
    const FUNC_NAME: &'static str = "list_acme_challenges";
    const DESCRIPTION: &'static str =
        "List the ACME HTTP-01 challenge files certbot currently has in the webroot (token + contents). These exist only while an issuance is mid-validation, so the list is usually empty and non-empty only during an init_http_01 / reissue_http_01 run.";
}

#[async_trait::async_trait]
impl McpToolCall<ListAcmeChallengesInputData, ListAcmeChallengesResponse>
    for ListAcmeChallengesHandler
{
    async fn execute_tool_call(
        &self,
        _model: ListAcmeChallengesInputData,
    ) -> Result<ListAcmeChallengesResponse, String> {
        let challenges = crate::scripts::list_acme_challenges().await?;

        Ok(ListAcmeChallengesResponse {
            challenges: challenges
                .into_iter()
                .map(|c| AcmeChallengeModel {
                    token: c.token,
                    content: c.content,
                })
                .collect(),
        })
    }
}
