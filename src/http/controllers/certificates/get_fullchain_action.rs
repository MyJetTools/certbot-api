use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;

use crate::app::AppContext;

#[http_route(
    method: "GET",
    route: "/api/certificates/v1/fullchain",
    summary: "Get Fullchain",
    description: "Get the fullchain certificate for a domain",
    controller: "Certificates",
    input_data: "GetFullchainInputModel",

    result:[
        {status_code: 200, description: "Fullchain retrieved successfully"},
        {status_code: 404, description: "Fullchain file not found"},
        {status_code: 500, description: "Failed to read fullchain file"},
    ]
)]
pub struct GetFullchainAction {
    _app: Arc<AppContext>,
}

impl GetFullchainAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

async fn handle_request(
    _action: &GetFullchainAction,
    input_data: GetFullchainInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = crate::scripts::get_fullchain(input_data.domain.as_str()).await;

    match result {
        Ok(content) => HttpOutput::as_text(content).into_ok_result(true).into(),
        Err(error) => {
            if error.contains("not found") {
                return HttpFailResult::as_not_found(error, false).into_err();
            }
            return HttpFailResult::as_fatal_error(error).into_err();
        }
    }
}

#[derive(MyHttpInput)]
pub struct GetFullchainInputModel {
    #[http_query(name = "domain", description = "Domain name")]
    pub domain: String,
}
