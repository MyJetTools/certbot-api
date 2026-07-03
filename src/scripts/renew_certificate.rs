use std::process::Stdio;

pub async fn renew_certificate(domain: String) -> Result<String, String> {
    let apex_domain = super::normalize_cert_name(&domain);
    let wildcard_domain = format!("*.{}", apex_domain);

    // certonly --expand instead of `certbot renew`: `renew` keeps whatever SANs
    // the certificate was originally issued with, while every cert must always
    // cover both the apex and the wildcard. certonly with both -d flags renews
    // when due (or when a SAN is missing) and re-issues with the full SAN set;
    // an up-to-date cert with both SANs is kept untouched.
    let mut cmd = tokio::process::Command::new("certbot");

    cmd.arg("certonly")
        .arg("--dns-cloudflare")
        .arg("--dns-cloudflare-credentials")
        .arg("/cloudflare.ini")
        .arg("--dns-cloudflare-propagation-seconds")
        .arg("30")
        .arg("--non-interactive")
        .arg("--cert-name")
        .arg(&apex_domain)
        .arg("--expand")
        .arg("-d")
        .arg(&apex_domain)
        .arg("-d")
        .arg(&wildcard_domain);

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to execute certbot: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Certbot renewal failed: {}", stderr))
    }
}
