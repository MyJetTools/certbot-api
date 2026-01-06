pub async fn get_private_key(domain: &str) -> Result<String, String> {
    let file_path = format!("/etc/letsencrypt/live/{}/privkey.pem", domain);

    match tokio::fs::read_to_string(&file_path).await {
        Ok(content) => Ok(content),
        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                Err(format!("Private key file not found for domain: {}", domain))
            } else {
                Err(format!("Failed to read private key file: {}", error))
            }
        }
    }
}
