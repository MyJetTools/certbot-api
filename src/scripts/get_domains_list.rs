pub async fn get_domains_list() -> Result<Vec<String>, String> {
    let live_dir = "/etc/letsencrypt/live";

    let mut entries = tokio::fs::read_dir(live_dir).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            format!("LetsEncrypt live directory not found: {}", live_dir)
        } else {
            format!("Failed to read directory {}: {}", live_dir, e)
        }
    })?;

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
