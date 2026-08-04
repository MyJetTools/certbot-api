pub async fn add_domain(domain: String, email: String) -> Result<String, String> {
    // Issue a single certificate that covers both the apex and the wildcard:
    //   "example.com"   -> SANs: example.com, *.example.com
    //   "*.example.com" -> SANs: example.com, *.example.com
    let apex_domain = super::normalize_cert_name(&domain);
    let wildcard_domain = format!("*.{}", apex_domain);

    // Reciprocal guard to the HTTP-01 no-clobber check: refuse to (re)issue a
    // DNS-01 apex+wildcard cert over a name that already belongs to an HTTP-01
    // (webroot) lineage, which this would silently destroy.
    if let Some(authenticator) = super::try_read_renewal_authenticator(&apex_domain)
        .await
        .map_err(|e| e.to_string())?
    {
        if authenticator == "webroot" {
            return Err(format!(
                "Certificate '{}' was issued via HTTP-01 (webroot). Use reissue_http_01 instead — add_domain would replace it with a DNS apex+wildcard cert.",
                apex_domain
            ));
        }
    }

    let mut cmd = tokio::process::Command::new("certbot");

    cmd.arg("certonly")
        .arg("--dns-cloudflare")
        .arg("--dns-cloudflare-credentials")
        .arg("/cloudflare.ini")
        .arg("--dns-cloudflare-propagation-seconds")
        .arg("30")
        .arg("--email")
        .arg(&email)
        .arg("--agree-tos")
        .arg("--non-interactive")
        .arg("--cert-name")
        .arg(&apex_domain)
        .arg("--expand")
        .arg("-d")
        .arg(&apex_domain)
        .arg("-d")
        .arg(&wildcard_domain);

    super::run_certbot(cmd, "Certbot failed").await
}
