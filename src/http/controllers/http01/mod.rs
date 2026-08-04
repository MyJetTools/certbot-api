pub mod init_action;
pub mod reissue_action;
pub mod status_action;
pub use init_action::*;
pub use reissue_action::*;
pub use status_action::*;

use crate::app::{StartTaskOutcome, StartedTask};

/// Build the HTTP response for a started (or already-running) HTTP-01 task.
pub fn start_http_01_response(started: &StartedTask) -> StartTaskHttpModel {
    let cert_name = &started.task.cert_name;
    let (status, message) = match started.outcome {
        StartTaskOutcome::Started => (
            "started",
            format!(
                "HTTP-01 task for '{}' started. Poll /api/http01/v1/status?domain={} to read the result.",
                cert_name, cert_name
            ),
        ),
        StartTaskOutcome::AlreadyRunning => (
            "already_running",
            format!(
                "A certificate task for '{}' is already in progress. Poll /api/http01/v1/status?domain={} to read the result.",
                cert_name, cert_name
            ),
        ),
    };

    StartTaskHttpModel {
        status: status.to_string(),
        task_id: started.task.id.clone(),
        cert_name: cert_name.clone(),
        message,
    }
}
