use std::process::Stdio;

pub async fn renew_certificate(domain: String) -> Result<String, String> {
    let mut cmd = tokio::process::Command::new("certbot");

    cmd.arg("renew")
        .arg("--cert-name")
        .arg(&domain)
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
