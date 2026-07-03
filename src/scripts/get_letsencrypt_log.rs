const LOG_FILE: &str = "/var/log/letsencrypt/letsencrypt.log";

pub async fn get_letsencrypt_log(tail_lines: Option<usize>) -> Result<String, String> {
    let content = tokio::fs::read_to_string(LOG_FILE).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            format!("Log file not found: {}", LOG_FILE)
        } else {
            format!("Failed to read {}: {}", LOG_FILE, e)
        }
    })?;

    let result = match tail_lines {
        Some(n) if n > 0 => {
            let lines: Vec<&str> = content.lines().collect();
            let start = lines.len().saturating_sub(n);
            lines[start..].join("\n")
        }
        _ => content,
    };

    Ok(result)
}
