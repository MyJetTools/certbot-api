use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;

use crate::app::AppContext;

#[http_route(
    method: "POST",
    route: "/api/certbot/v1/renew-certificate",
    summary: "Renew Certificate",
    description: "Renew a certificate for a specific domain",
    controller: "CertBot",
    input_data: "RenewCertificateInputModel",

    result:[
        {status_code: 200, description: "Certificate renewed successfully"},
        {status_code: 404, description: "Domain certificate not found"},
        {status_code: 500, description: "Failed to renew certificate"},
    ]
)]
pub struct RenewCertificateAction {
    _app: Arc<AppContext>,
}

impl RenewCertificateAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

async fn handle_request(
    _action: &RenewCertificateAction,
    input_data: RenewCertificateInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = crate::scripts::renew_certificate(input_data.domain).await;

    match result {
        Ok(output) => HttpOutput::as_text(output).into_ok_result(true).into(),
        Err(error) => {
            if error.contains("not found") || error.contains("does not exist") {
                return HttpFailResult::as_not_found(error, false).into_err();
            }
            return HttpFailResult::as_fatal_error(error).into_err();
        }
    }
}

#[derive(MyHttpInput)]
pub struct RenewCertificateInputModel {
    #[http_body(name = "domain", description = "Domain name to renew certificate for")]
    pub domain: String,
}
