use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;

use crate::app::AppContext;

#[http_route(
    method: "GET",
    route: "/api/certificates/v1/{domain}/private-key",
    summary: "Get Private Key",
    description: "Get the private key for a domain certificate",
    controller: "Certificates",
    input_data: "GetPrivateKeyInputModel",

    result:[
        {status_code: 200, description: "Private key retrieved successfully"},
        {status_code: 404, description: "Private key file not found"},
        {status_code: 500, description: "Failed to read private key file"},
    ]
)]
pub struct GetPrivateKeyAction {
    _app: Arc<AppContext>,
}

impl GetPrivateKeyAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

async fn handle_request(
    _action: &GetPrivateKeyAction,
    input_data: GetPrivateKeyInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = crate::scripts::get_private_key(input_data.domain).await;

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
pub struct GetPrivateKeyInputModel {
    #[http_path(name = "domain", description = "Domain name")]
    pub domain: String,
}
