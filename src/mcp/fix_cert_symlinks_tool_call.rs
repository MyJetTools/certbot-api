use std::sync::Arc;

use mcp_server_middleware::*;

use serde::*;

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct FixCertSymlinksInputData {
    #[property(description = "Domain name whose /etc/letsencrypt/live directory should be repaired")]
    pub domain: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct FixedFileItem {
    #[property(description = "File name inside the live directory (cert.pem, chain.pem, fullchain.pem, privkey.pem)")]
    pub file: String,

    #[property(
        description = "What was done: 'already_valid' (symlink was correct), 'relinked' (symlink pointed elsewhere), 'backed_up_and_linked' (regular file moved to backup, symlink created), 'created' (file was missing, symlink created)"
    )]
    pub action: String,

    #[property(description = "Where the replaced regular file was moved, if any")]
    pub backup_path: Option<String>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct FixCertSymlinksResponse {
    #[property(description = "Normalized certificate name (wildcard prefix stripped)")]
    pub domain: String,

    #[property(description = "Archive version N the symlinks now point to (certN.pem)")]
    pub archive_version: u32,

    #[property(description = "Per-file repair report")]
    pub files: Vec<FixedFileItem>,
}

pub struct FixCertSymlinksHandler {
    _app: Arc<AppContext>,
}

impl FixCertSymlinksHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

impl ToolDefinition for FixCertSymlinksHandler {
    const FUNC_NAME: &'static str = "fix_cert_symlinks";
    const DESCRIPTION: &'static str =
        "Repair a broken certbot live directory. Certbot requires the .pem files in /etc/letsencrypt/live/<domain>/ to be symlinks into the archive directory; if they were replaced by regular files, renewal fails with 'expected ... to be a symlink'. This tool backs up the regular files and recreates symlinks to the latest archive version.";
}

#[async_trait::async_trait]
impl McpToolCall<FixCertSymlinksInputData, FixCertSymlinksResponse> for FixCertSymlinksHandler {
    async fn execute_tool_call(
        &self,
        model: FixCertSymlinksInputData,
    ) -> Result<FixCertSymlinksResponse, String> {
        let result = crate::scripts::fix_cert_symlinks(model.domain).await?;

        let files = result
            .files
            .into_iter()
            .map(|file| FixedFileItem {
                file: file.file,
                action: file.action,
                backup_path: file.backup_path,
            })
            .collect();

        Ok(FixCertSymlinksResponse {
            domain: result.domain,
            archive_version: result.archive_version,
            files,
        })
    }
}
