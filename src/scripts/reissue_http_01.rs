use super::CertbotPrepareError;

/// Everything a validated HTTP-01 reissue needs to run, produced by
/// [`prepare_reissue_http_01`] and consumed by [`run_reissue_http_01`].
pub struct PreparedHttp01Reissue {
    pub cert_name: String,
}

/// Validate a reissue request. Fast (one small config read), so it runs
/// synchronously at job-start time and reports a bad request immediately.
///
/// We verify the lineage was actually issued via `webroot`. A cert issued with a
/// different authenticator (e.g. a DNS-01 wildcard) must never be driven down the
/// HTTP-01 path — certbot would otherwise silently use its *stored* method, so
/// the guard is about honesty of this endpoint, not about certbot failing.
pub async fn prepare_reissue_http_01(
    domain: String,
) -> Result<PreparedHttp01Reissue, CertbotPrepareError> {
    let cert_name = domain.trim().to_string();

    if cert_name.is_empty() {
        return Err(CertbotPrepareError::Invalid(
            "A domain (certificate name) is required".to_string(),
        ));
    }

    if cert_name.starts_with("*.") {
        return Err(CertbotPrepareError::Invalid(
            "HTTP-01 certificates are per-FQDN; pass the exact primary domain, not a wildcard."
                .to_string(),
        ));
    }

    match super::try_read_renewal_authenticator(&cert_name).await? {
        None => Err(CertbotPrepareError::NotFound(format!(
            "Renewal config for '{}' not found. Issue it first with init_http_01.",
            cert_name
        ))),
        Some(authenticator) if authenticator != "webroot" => {
            Err(CertbotPrepareError::Invalid(format!(
                "Certificate '{}' was issued with authenticator '{}', not 'webroot'. Use renew_certificate for the DNS-01 flow instead.",
                cert_name, authenticator
            )))
        }
        Some(_) => Ok(PreparedHttp01Reissue { cert_name }),
    }
}

/// Force-reissue the certificate now. Long-running, so this is what the
/// background task awaits; serialized and timed out by [`super::run_certbot`].
///
/// `certbot renew --cert-name <name>` restores the stored domain set and webroot
/// path from the renewal config, so we pass only the lineage name.
/// `--force-renewal` makes it reissue immediately instead of skipping a cert that
/// is not yet due.
pub async fn run_reissue_http_01(prepared: PreparedHttp01Reissue) -> Result<String, String> {
    let PreparedHttp01Reissue { cert_name } = prepared;

    let mut cmd = tokio::process::Command::new("certbot");

    cmd.arg("renew")
        .arg("--cert-name")
        .arg(&cert_name)
        .arg("--force-renewal")
        .arg("--non-interactive");

    super::run_certbot(cmd, "Certbot HTTP-01 reissue failed").await
}
