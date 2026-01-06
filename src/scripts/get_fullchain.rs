pub async fn get_fullchain(domain: String) -> Result<String, String> {
    let file_path = format!("/etc/letsencrypt/live/{}/fullchain.pem", domain);

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
