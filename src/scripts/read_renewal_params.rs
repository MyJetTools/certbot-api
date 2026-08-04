use super::CertbotPrepareError;

/// Certbot records how a certificate was issued in
/// `/etc/letsencrypt/renewal/<cert-name>.conf`, section `[renewalparams]`, key
/// `authenticator` (e.g. `webroot`, `dns-cloudflare`, `standalone`). This is the
/// single source of truth for "which challenge method owns this lineage" — no
/// separate settings store is needed, and `certbot renew` reads the same file.
///
/// Returns:
/// - `Ok(None)` when the lineage has **no** renewal config at all (never issued);
/// - `Ok(Some(authenticator))` when the config exists and names one;
/// - `Err(Io)` on a real IO failure, or when the config exists but is malformed
///   (no `authenticator` entry). The malformed case is an error rather than
///   `None` on purpose: a caller's no-clobber guard must not silently treat a
///   config it failed to understand as "no config" and proceed.
pub async fn try_read_renewal_authenticator(
    cert_name: &str,
) -> Result<Option<String>, CertbotPrepareError> {
    let path = format!("/etc/letsencrypt/renewal/{}.conf", cert_name);

    let content = match tokio::fs::read_to_string(&path).await {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(CertbotPrepareError::Io(format!("Failed to read {}: {}", path, e))),
    };

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        // Entries look like `authenticator = webroot`.
        if let Some((key, value)) = line.split_once('=') {
            if key.trim() == "authenticator" {
                return Ok(Some(value.trim().to_string()));
            }
        }
    }

    Err(CertbotPrepareError::Io(format!(
        "Renewal config {} has no 'authenticator' entry",
        path
    )))
}
