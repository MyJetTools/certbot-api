/// Serve the raw ACME HTTP-01 challenge file that certbot wrote into the webroot
/// (`<webroot>/.well-known/acme-challenge/<token>`). Returns the file contents,
/// or `None` when it does not exist (no active challenge for that token).
pub async fn read_acme_challenge(token: &str) -> Result<Option<String>, String> {
    // ACME tokens are base64url (RFC 8555): [A-Za-z0-9_-]. Reject anything else
    // so a crafted token can never escape the challenge directory.
    if token.is_empty()
        || !token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Ok(None);
    }

    let path = format!(
        "{}/.well-known/acme-challenge/{}",
        super::DEFAULT_HTTP_01_WEBROOT,
        token
    );

    match tokio::fs::read_to_string(&path).await {
        Ok(content) => Ok(Some(content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Failed to read {}: {}", path, e)),
    }
}
