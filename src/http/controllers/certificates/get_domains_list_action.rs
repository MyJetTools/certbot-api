use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[http_route(
    method: "GET",
    route: "/api/certificates/v1/domains",
    summary: "Get Domains List",
    description: "Get list of all domains from",
    controller: "Certificates",

    result:[
        {status_code: 200, description: "Domains list retrieved successfully", model: DomainsListHttpModel},
        {status_code: 500, description: "Failed to read domains list"},
    ]
)]
pub struct GetDomainsListAction {
    _app: Arc<AppContext>,
}

impl GetDomainsListAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

async fn handle_request(
    _action: &GetDomainsListAction,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = crate::scripts::get_domains_list().await;

    match result {
        Ok(domains) => {
            let result = DomainsListHttpModel { domains };
            HttpOutput::as_json(result).into_ok_result(true).into()
        }
        Err(error) => HttpFailResult::as_fatal_error(error).into_err(),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct DomainsListHttpModel {
    pub domains: Vec<String>,
}
