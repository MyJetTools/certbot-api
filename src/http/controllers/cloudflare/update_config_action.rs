use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;

use crate::app::AppContext;

#[http_route(
    method: "POST",
    route: "/api/cloudflare/v1/update-config",
    summary: "Update Cloudflare Config",
    description: "Update /cloudflare.ini file with new API token",
    controller: "Cloudflare",
    input_data: "UpdateCloudflareConfigInputModel",

    result:[
        {status_code: 200, description: "Config updated successfully"},
    ]
)]
pub struct UpdateCloudflareConfigAction {
    _app: Arc<AppContext>,
}

impl UpdateCloudflareConfigAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

async fn handle_request(
    _action: &UpdateCloudflareConfigAction,
    input_data: UpdateCloudflareConfigInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::scripts::update_cloudflare_config(input_data.api_token).await;

    HttpOutput::Empty.into_ok_result(true).into()
}

#[derive(MyHttpInput)]
pub struct UpdateCloudflareConfigInputModel {
    #[http_body(name = "api_token", description = "Cloudflare API token")]
    pub api_token: String,
}
