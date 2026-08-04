use serde::Deserialize;

/// Mirrors the server's `TaskHttpModel` (an item of `GET /api/tasks/v1/list`).
///
/// Every optional field carries `#[serde(default)]` so an older or newer server
/// still deserializes cleanly instead of failing the whole list.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct TaskModel {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub cert_name: String,
    #[serde(default)]
    pub domains: Vec<String>,
    /// `issue-http-01` | `reissue-http-01` | `renew-dns`.
    #[serde(default)]
    pub kind: String,
    /// `running` | `completed` | `failed`.
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    /// rfc3339, e.g. `2026-07-16T12:04:31.123456+00:00`.
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

impl TaskModel {
    pub fn is_running(&self) -> bool {
        self.status == "running"
    }

    /// Just the clock part of the created_at rfc3339 stamp: `12:04:31`.
    pub fn time_label(&self) -> &str {
        let Some(t_pos) = self.created_at.find('T') else {
            return self.created_at.as_str();
        };
        let rest = &self.created_at[t_pos + 1..];
        rest.get(..8).unwrap_or(rest)
    }

    pub fn domains_label(&self) -> String {
        self.domains.join(", ")
    }

    /// Hover text for the status pill — certbot's error / output tail.
    pub fn status_title(&self) -> Option<String> {
        if let Some(error) = &self.error {
            return Some(error.clone());
        }
        self.output.as_ref().map(|o| {
            // Show only the tail of a long certbot stdout as a tooltip.
            let trimmed = o.trim();
            let start = trimmed.len().saturating_sub(400);
            trimmed[start..].to_string()
        })
    }
}

/// Envelope of `GET /api/tasks/v1/list` — `{"tasks": [...]}`, newest first.
#[derive(Deserialize, Default)]
pub struct TasksResponse {
    #[serde(default)]
    pub tasks: Vec<TaskModel>,
}
