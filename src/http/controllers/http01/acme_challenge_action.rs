use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;

use crate::app::AppContext;

#[http_route(
    method: "GET",
    route: "/.well-known/acme-challenge/{token}",
    summary: "Serve ACME HTTP-01 Challenge",
    description: "Serve the raw ACME HTTP-01 challenge file certbot wrote into the webroot. Let's Encrypt fetches this over :80 during an init_http_01 / reissue_http_01 run; point the reverse proxy's /.well-known/acme-challenge/ path here.",
    controller: "Http01",
    input_data: "AcmeChallengeInputModel",

    result:[
        {status_code: 200, description: "Challenge token contents"},
        {status_code: 404, description: "No active challenge for this token"},
    ]
)]
pub struct AcmeChallengeAction {
    _app: Arc<AppContext>,
}

impl AcmeChallengeAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

async fn handle_request(
    _action: &AcmeChallengeAction,
    input_data: AcmeChallengeInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    match crate::scripts::read_acme_challenge(&input_data.token).await {
        Ok(Some(content)) => HttpOutput::as_text(content).into_ok_result(true).into(),
        Ok(None) => HttpFailResult::as_not_found("No active challenge for this token".to_string(), false).into_err(),
        Err(error) => HttpFailResult::as_fatal_error(error).into_err(),
    }
}

#[derive(MyHttpInput)]
pub struct AcmeChallengeInputModel {
    #[http_path(name = "token", description = "ACME challenge token (the last path segment)")]
    pub token: String,
}
