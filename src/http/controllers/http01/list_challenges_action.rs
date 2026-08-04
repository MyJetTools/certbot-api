use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[http_route(
    method: "GET",
    route: "/api/http01/v1/challenges",
    summary: "List Active ACME Challenges",
    description: "List every ACME HTTP-01 challenge file certbot currently has in the webroot: the token (last URL segment) and its contents. These exist only while an issuance is mid-validation, so this is usually empty and non-empty only during an init/reissue run.",
    controller: "Http01",

    result:[
        {status_code: 200, description: "Current challenges", model: AcmeChallengesHttpModel},
        {status_code: 500, description: "Failed to read the webroot"},
    ]
)]
pub struct ListChallengesAction {
    _app: Arc<AppContext>,
}

impl ListChallengesAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

async fn handle_request(
    _action: &ListChallengesAction,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    match crate::scripts::list_acme_challenges().await {
        Ok(challenges) => {
            let result = AcmeChallengesHttpModel {
                challenges: challenges
                    .into_iter()
                    .map(|c| AcmeChallengeHttpModel {
                        token: c.token,
                        content: c.content,
                    })
                    .collect(),
            };
            HttpOutput::as_json(result).into_ok_result(true).into()
        }
        Err(error) => HttpFailResult::as_fatal_error(error).into_err(),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct AcmeChallengeHttpModel {
    pub token: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct AcmeChallengesHttpModel {
    pub challenges: Vec<AcmeChallengeHttpModel>,
}
