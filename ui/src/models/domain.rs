use serde::Deserialize;

/// Mirrors the server's `DomainExpirationHttpItem`
/// (an item of `GET /api/certificates/v1/domains-expiration`).
///
/// Every optional field carries `#[serde(default)]` so an older or newer server
/// still deserializes cleanly instead of failing the whole list.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct DomainModel {
    #[serde(default)]
    pub domain: String,
    /// rfc3339 expiry, e.g. `2026-07-16T12:04:31+00:00`. Absent when the cert
    /// could not be read (see `error`).
    #[serde(default)]
    pub expiration: Option<String>,
    /// Present when the cert for this domain could not be read.
    #[serde(default)]
    pub error: Option<String>,
}

impl DomainModel {
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// Just the date part of the rfc3339 stamp: `2026-07-16`.
    pub fn expiration_label(&self) -> String {
        match &self.expiration {
            Some(exp) => exp.split('T').next().unwrap_or(exp).to_string(),
            None => "—".to_string(),
        }
    }

    pub fn status_label(&self) -> &'static str {
        if self.is_error() { "error" } else { "valid" }
    }

    /// Hover text for the status pill — the read error, if any.
    pub fn error_title(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

/// Envelope of `GET /api/certificates/v1/domains-expiration` — `{"domains": [...]}`.
#[derive(Deserialize, Default)]
pub struct DomainsResponse {
    #[serde(default)]
    pub domains: Vec<DomainModel>,
}
