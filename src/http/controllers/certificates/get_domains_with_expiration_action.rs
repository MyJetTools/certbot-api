use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[http_route(
    method: "GET",
    route: "/api/certificates/v1/domains-expiration",
    summary: "Get Domains with Expiration",
    description: "Get list of all domains with their expiration dates",
    controller: "Certificates",

    result:[
        {status_code: 200, description: "Domains with expiration retrieved successfully", model: DomainsExpirationHttpModel},
        {status_code: 500, description: "Failed to read domains list"},
    ]
)]
pub struct GetDomainsWithExpirationAction {
    _app: Arc<AppContext>,
}

impl GetDomainsWithExpirationAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

async fn handle_request(
    _action: &GetDomainsWithExpirationAction,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = crate::scripts::get_domains_with_expiration().await;

    match result {
        Ok(domains_expiration) => {
            let items: Vec<DomainExpirationHttpItem> = domains_expiration
                .into_iter()
                .map(|item| DomainExpirationHttpItem {
                    domain: item.domain,
                    expiration: item.expiration,
                    error: item.error,
                })
                .collect();

            let result = DomainsExpirationHttpModel { domains: items };
            HttpOutput::as_json(result).into_ok_result(true).into()
        }
        Err(error) => HttpFailResult::as_fatal_error(error).into_err(),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct DomainsExpirationHttpModel {
    pub domains: Vec<DomainExpirationHttpItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct DomainExpirationHttpItem {
    pub domain: String,
    pub expiration: Option<String>,
    pub error: Option<String>,
}
