pub async fn get_domains_list() -> Result<Vec<String>, String> {
    let live_dir = "/etc/letsencrypt/live";

    // A host that has never issued a certificate has no live directory yet —
    // that is an empty list, not an error.
    let mut entries = match tokio::fs::read_dir(live_dir).await {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("Failed to read directory {}: {}", live_dir, e)),
    };

    let mut domains = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read directory entry: {}", e))?
    {
        let path = entry.path();

        // Only include directories, skip files
        if path.is_dir() {
            if let Some(domain_name) = path.file_name() {
                if let Some(domain_str) = domain_name.to_str() {
                    // Skip the README file if it exists as a directory (unlikely but safe)
                    if domain_str != "README" {
                        domains.push(domain_str.to_string());
                    }
                }
            }
        }
    }

    domains.sort();
    Ok(domains)
}
