use std::process::Stdio;

pub async fn renew_certificate(domain: String) -> Result<String, String> {
    // Normalize "*.example.com" -> "example.com": certbot stores the cert under
    // the apex name regardless of whether the original request was wildcard.
    let cert_name = domain
        .strip_prefix("*.")
        .map(|s| s.to_string())
        .unwrap_or(domain);

    let mut cmd = tokio::process::Command::new("certbot");

    cmd.arg("renew")
        .arg("--cert-name")
        .arg(&cert_name)
        .arg("--dns-cloudflare")
        .arg("--dns-cloudflare-credentials")
        .arg("/cloudflare.ini")
        .arg("--dns-cloudflare-propagation-seconds")
        .arg("30")
        .arg("--non-interactive");

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
