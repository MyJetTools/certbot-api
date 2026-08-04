/// A single ACME HTTP-01 challenge currently present in the webroot: the token
/// (the file name / last URL segment) and its contents (the key authorization).
pub struct AcmeChallenge {
    pub token: String,
    pub content: String,
}

/// List every challenge file certbot currently has in the webroot
/// (`<webroot>/.well-known/acme-challenge/`). These exist only while an issuance
/// is mid-validation, so the list is usually empty and non-empty only during an
/// `init_http_01` / `reissue_http_01` run. An absent directory is an empty list.
pub async fn list_acme_challenges() -> Result<Vec<AcmeChallenge>, String> {
    let dir = format!(
        "{}/.well-known/acme-challenge",
        super::DEFAULT_HTTP_01_WEBROOT
    );

    let mut entries = match tokio::fs::read_dir(&dir).await {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("Failed to read {}: {}", dir, e)),
    };

    let mut result = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read directory entry: {}", e))?
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let token = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let content = tokio::fs::read_to_string(&path)
            .await
            .unwrap_or_default()
            .trim_end()
            .to_string();

        result.push(AcmeChallenge { token, content });
    }

    result.sort_by(|a, b| a.token.cmp(&b.token));
    Ok(result)
}
