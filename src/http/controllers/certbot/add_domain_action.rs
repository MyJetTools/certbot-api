use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;

use crate::app::AppContext;

#[http_route(
    method: "POST",
    route: "/api/certbot/v1/add-domain",
    summary: "Add Domain Certificate",
    description: "Add a new domain certificate using certbot with Cloudflare DNS",
    controller: "CertBot",
    input_data: "AddDomainInputModel",

    result:[
        {status_code: 200, description: "Domain certificate added successfully"},
        {status_code: 500, description: "Failed to add domain certificate"},
    ]
)]
pub struct AddDomainAction {
    _app: Arc<AppContext>,
}

impl AddDomainAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

async fn handle_request(
    _action: &AddDomainAction,
    input_data: AddDomainInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = crate::scripts::add_domain(input_data.domain, input_data.email).await;

    match result {
        Ok(output) => HttpOutput::as_text(output).into_ok_result(true).into(),
        Err(error) => {
            // Return error as JSON response
            return HttpFailResult::as_fatal_error(error).into_err();
        }
    }
}

#[derive(MyHttpInput)]
pub struct AddDomainInputModel {
    #[http_body(name = "domain", description = "Domain name to add certificate for")]
    pub domain: String,

    #[http_body(
        name = "email",
        description = "Email address for certificate registration"
    )]
    pub email: String,
}
