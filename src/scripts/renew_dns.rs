use super::CertbotPrepareError;

/// Everything a validated DNS-01 renewal needs to run, produced by
/// [`prepare_renew_dns`] and consumed by [`run_renew_dns`].
pub struct PreparedRenewDns {
    pub cert_name: String,
    pub domains: Vec<String>,
}

/// Validate a DNS-01 renewal request. Keyed by the normalized apex, every DNS
/// cert always covers both the apex and its wildcard.
///
/// Reciprocal guard to the HTTP-01 no-clobber check: refuse to run a DNS-01
/// renewal against a lineage that was issued via `webroot`. `certonly
/// --dns-cloudflare --cert-name <name> --expand` would otherwise reconfigure that
/// lineage in place — flipping its authenticator to dns-cloudflare and replacing
/// its SAN set with apex+wildcard — silently destroying the HTTP-01 certificate.
pub async fn prepare_renew_dns(domain: String) -> Result<PreparedRenewDns, CertbotPrepareError> {
    let apex = super::normalize_cert_name(&domain);

    if apex.is_empty() {
        return Err(CertbotPrepareError::Invalid("A domain is required".to_string()));
    }

    if let Some(authenticator) = super::try_read_renewal_authenticator(&apex).await? {
        if authenticator == "webroot" {
            return Err(CertbotPrepareError::Invalid(format!(
                "Certificate '{}' was issued via HTTP-01 (webroot). Use reissue_http_01 instead — a DNS-01 renewal would replace it with a DNS apex+wildcard cert.",
                apex
            )));
        }
    }

    let wildcard = format!("*.{}", apex);

    Ok(PreparedRenewDns {
        cert_name: apex.clone(),
        domains: vec![apex, wildcard],
    })
}

/// Run certbot to renew/reissue the DNS-01 certificate. Long-running (DNS
/// propagation), so this is what the background task awaits; serialized and timed
/// out by [`super::run_certbot`].
///
/// `certonly --expand` instead of `certbot renew`: `renew` keeps whatever SANs
/// the certificate was originally issued with, while every DNS cert must always
/// cover both the apex and the wildcard. certonly with both -d flags renews when
/// due (or when a SAN is missing) and re-issues with the full SAN set.
pub async fn run_renew_dns(prepared: PreparedRenewDns) -> Result<String, String> {
    let PreparedRenewDns { cert_name, domains } = prepared;

    let mut cmd = tokio::process::Command::new("certbot");

    cmd.arg("certonly")
        .arg("--dns-cloudflare")
        .arg("--dns-cloudflare-credentials")
        .arg("/cloudflare.ini")
        .arg("--dns-cloudflare-propagation-seconds")
        .arg("30")
        .arg("--non-interactive")
        .arg("--cert-name")
        .arg(&cert_name)
        .arg("--expand");

    for domain in &domains {
        cmd.arg("-d").arg(domain);
    }

    super::run_certbot(cmd, "Certbot renewal failed").await
}
