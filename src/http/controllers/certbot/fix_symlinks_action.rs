use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[http_route(
    method: "POST",
    route: "/api/certbot/v1/fix-symlinks",
    summary: "Fix Certificate Symlinks",
    description: "Repair /etc/letsencrypt/live/<domain>: certbot requires cert.pem, chain.pem, fullchain.pem and privkey.pem to be symlinks into the archive directory. If they were replaced by regular files (copy without -a, scp, docker cp), renewal fails with 'expected ... to be a symlink'. This action moves regular files to a backup directory and recreates symlinks to the latest archive version.",
    controller: "CertBot",
    input_data: "FixSymlinksInputModel",

    result:[
        {status_code: 200, description: "Symlinks repaired", model: FixSymlinksHttpModel},
        {status_code: 404, description: "Archive directory or archive files not found"},
        {status_code: 500, description: "Failed to repair symlinks"},
    ]
)]
pub struct FixSymlinksAction {
    _app: Arc<AppContext>,
}

impl FixSymlinksAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

async fn handle_request(
    _action: &FixSymlinksAction,
    input_data: FixSymlinksInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = crate::scripts::fix_cert_symlinks(input_data.domain).await;

    match result {
        Ok(fix_result) => {
            let response = FixSymlinksHttpModel {
                domain: fix_result.domain,
                archive_version: fix_result.archive_version,
                files: fix_result
                    .files
                    .into_iter()
                    .map(|file| FixedFileHttpModel {
                        file: file.file,
                        action: file.action,
                        backup_path: file.backup_path,
                    })
                    .collect(),
            };
            HttpOutput::as_json(response).into_ok_result(true).into()
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
pub struct FixSymlinksInputModel {
    #[http_body(
        name = "domain",
        description = "Domain name whose live directory should be repaired"
    )]
    pub domain: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct FixSymlinksHttpModel {
    pub domain: String,
    pub archive_version: u32,
    pub files: Vec<FixedFileHttpModel>,
}

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct FixedFileHttpModel {
    pub file: String,
    pub action: String,
    pub backup_path: Option<String>,
}
