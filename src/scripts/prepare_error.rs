use std::fmt;

/// Why a certbot operation could not even be started. Kept as a typed value so
/// callers map it to the right HTTP status (404 / 400 / 500) instead of
/// substring-matching prose.
#[derive(Debug, Clone)]
pub enum CertbotPrepareError {
    /// The lineage / renewal config the request refers to does not exist. → 404.
    NotFound(String),
    /// The request itself is malformed (empty/wildcard domain, wrong
    /// authenticator for this flow, …). → 400.
    Invalid(String),
    /// A server-side failure while preparing (IO error reading a renewal config,
    /// a malformed config, …). → 500.
    Io(String),
}

impl CertbotPrepareError {
    pub fn message(&self) -> &str {
        match self {
            CertbotPrepareError::NotFound(m) => m,
            CertbotPrepareError::Invalid(m) => m,
            CertbotPrepareError::Io(m) => m,
        }
    }
}

impl fmt::Display for CertbotPrepareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}
