use std::process::Stdio;

pub async fn add_domain(domain: String, email: String) -> Result<String, String> {
    // Issue a single certificate that covers both the apex and the wildcard:
    //   "example.com"   -> SANs: example.com, *.example.com
    //   "*.example.com" -> SANs: example.com, *.example.com
    let apex_domain = domain
        .strip_prefix("*.")
        .map(|s| s.to_string())
        .unwrap_or(domain);
    let wildcard_domain = format!("*.{}", apex_domain);

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
        Err(format!("Certbot failed: {}", stderr))
    }
}
