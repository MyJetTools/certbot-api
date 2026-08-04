use super::CertbotPrepareError;

/// Directory certbot writes ACME HTTP-01 challenge files into. Whatever owns
/// port 80 on this host (e.g. the reverse proxy) must serve
/// `<webroot>/.well-known/acme-challenge/` publicly for the validation to pass.
pub const DEFAULT_HTTP_01_WEBROOT: &str = "/var/www/acme";

/// Everything a validated HTTP-01 issuance needs to run, produced by
/// [`prepare_init_http_01`] and consumed by [`run_init_http_01`].
pub struct PreparedHttp01Issue {
    pub cert_name: String,
    pub domains: Vec<String>,
    pub email: String,
    pub webroot: String,
}

/// Validate an issuance request and reserve the lineage name. Everything here is
/// fast (input checks + one small config read), so it runs synchronously at
/// job-start time and reports a bad request immediately — only the certbot call
/// itself is deferred to the background job.
///
/// Unlike the DNS-01 flow, HTTP-01 validates each exact FQDN by fetching a token
/// over :80, so it cannot cover a wildcard — every hostname must be listed. The
/// certificate lineage is named after the first (primary) domain, verbatim.
pub async fn prepare_init_http_01(
    domains: Vec<String>,
    email: String,
    webroot: Option<String>,
) -> Result<PreparedHttp01Issue, CertbotPrepareError> {
    let webroot = webroot
        .map(|w| w.trim().to_string())
        .filter(|w| !w.is_empty())
        .unwrap_or_else(|| DEFAULT_HTTP_01_WEBROOT.to_string());

    let email = email.trim().to_string();
    if email.is_empty() {
        return Err(CertbotPrepareError::Invalid(
            "Email is required for ACME registration".to_string(),
        ));
    }

    let domains: Vec<String> = domains
        .into_iter()
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty())
        .collect();

    if domains.is_empty() {
        return Err(CertbotPrepareError::Invalid(
            "At least one domain is required".to_string(),
        ));
    }

    if let Some(wildcard) = domains.iter().find(|d| d.starts_with("*.")) {
        return Err(CertbotPrepareError::Invalid(format!(
            "HTTP-01 cannot validate the wildcard domain '{}'. Wildcards require DNS-01 — use add_domain.",
            wildcard
        )));
    }

    // The lineage is named after the first domain, exactly as given — HTTP-01
    // certs are per-FQDN, so no apex normalization happens here.
    let cert_name = domains[0].clone();

    // Guard against clobbering a lineage that was issued a different way: reusing
    // its name with `certonly --webroot` would flip its authenticator to webroot
    // and drop its current SAN set (e.g. a DNS-01 apex+wildcard cert).
    if let Some(authenticator) = super::try_read_renewal_authenticator(&cert_name).await? {
        if authenticator != "webroot" {
            return Err(CertbotPrepareError::Invalid(format!(
                "A certificate named '{}' already exists with authenticator '{}'. Refusing to overwrite it via HTTP-01 (it would drop its current SANs). Use a different primary domain, or the matching renewal flow.",
                cert_name, authenticator
            )));
        }
    }

    Ok(PreparedHttp01Issue {
        cert_name,
        domains,
        email,
        webroot,
    })
}

/// Run certbot to issue the certificate. Long-running (network + ACME), so this
/// is what the background task awaits; serialized against every other certbot
/// invocation and killed after the timeout by [`super::run_certbot`].
pub async fn run_init_http_01(prepared: PreparedHttp01Issue) -> Result<String, String> {
    let PreparedHttp01Issue {
        cert_name,
        domains,
        email,
        webroot,
    } = prepared;

    let mut cmd = tokio::process::Command::new("certbot");

    cmd.arg("certonly")
        .arg("--webroot")
        .arg("-w")
        .arg(&webroot)
        .arg("--email")
        .arg(&email)
        .arg("--agree-tos")
        .arg("--non-interactive")
        .arg("--cert-name")
        .arg(&cert_name)
        .arg("--expand");

    for domain in &domains {
        cmd.arg("-d").arg(domain);
    }

    super::run_certbot(cmd, "Certbot HTTP-01 issuance failed").await
}
