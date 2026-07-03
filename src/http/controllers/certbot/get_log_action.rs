use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;

use crate::app::AppContext;

#[http_route(
    method: "GET",
    route: "/api/certbot/v1/log",
    summary: "Get LetsEncrypt Log",
    description: "Return the contents of /var/log/letsencrypt/letsencrypt.log as plain text. Use tailLines to limit the output to the last N lines.",
    controller: "CertBot",
    input_data: "GetLogInputModel",

    result:[
        {status_code: 200, description: "Log content"},
        {status_code: 404, description: "Log file not found"},
        {status_code: 500, description: "Failed to read log file"},
    ]
)]
pub struct GetLogAction {
    _app: Arc<AppContext>,
}

impl GetLogAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

async fn handle_request(
    _action: &GetLogAction,
    input_data: GetLogInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let tail_lines = input_data.tail_lines.map(|n| n as usize);

    let result = crate::scripts::get_letsencrypt_log(tail_lines).await;

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
pub struct GetLogInputModel {
    #[http_query(
        name = "tailLines",
        description = "Return only the last N lines of the log. Omit to get the whole file."
    )]
    pub tail_lines: Option<u32>,
}
