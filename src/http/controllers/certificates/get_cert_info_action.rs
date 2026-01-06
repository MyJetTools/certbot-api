use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[http_route(
    method: "GET",
    route: "/api/certificates/v1/info",
    summary: "Get Certificate Info",
    description: "Get certificate information for a domain",
    controller: "Certificates",
    input_data: "GetCertInfoInputModel",

    result:[
        {status_code: 200, description: "Certificate info retrieved successfully", model: CertificateInfoHttpModel},
        {status_code: 404, description: "Certificate files not found"},
        {status_code: 500, description: "Failed to read certificate info"},
    ]
)]
pub struct GetCertInfoAction {
    _app: Arc<AppContext>,
}

impl GetCertInfoAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

async fn handle_request(
    _action: &GetCertInfoAction,
    input_data: GetCertInfoInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = crate::scripts::get_cert_info(input_data.domain.as_str()).await;

    match result {
        Ok(cert_info) => {
            let result = CertificateInfoHttpModel {
                cn: cert_info.cn,
                expires: cert_info.expires.to_rfc3339(),
            };
            HttpOutput::as_json(result).into_ok_result(true).into()
        }
        Err(error) => {
            if error.contains("not found") {
                return HttpFailResult::as_not_found(error, false).into_err();
            }
            return HttpFailResult::as_fatal_error(error).into_err();
        }
    }
}

#[derive(MyHttpInput)]
pub struct GetCertInfoInputModel {
    #[http_query(name = "domain", description = "Domain name")]
    pub domain: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct CertificateInfoHttpModel {
    pub cn: String,
    pub expires: String,
}
