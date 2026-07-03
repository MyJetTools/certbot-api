pub async fn get_fullchain(domain: &str) -> Result<String, String> {
    let cert_name = super::normalize_cert_name(domain);
    let file_path = format!("/etc/letsencrypt/live/{}/fullchain.pem", cert_name);

    match tokio::fs::read_to_string(&file_path).await {
        Ok(content) => Ok(content),
        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                Err(format!("Fullchain file not found for domain: {}", domain))
            } else {
                Err(format!("Failed to read fullchain file: {}", error))
            }
        }
    }
}
