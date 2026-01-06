use std::process::Stdio;

pub async fn add_domain(domain: String, email: String) -> Result<String, String> {
    // If domain doesn't already start with *., prepend *. to make it a wildcard certificate
    // e.g., "domain.com" becomes "*.domain.com"
    let wildcard_domain = if domain.starts_with("*.") {
        domain
    } else {
        format!("*.{}", domain)
    };

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
