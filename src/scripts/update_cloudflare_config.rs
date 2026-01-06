pub async fn update_cloudflare_config(api_token: String) {
    let config_content = format!(
        "# Cloudflare API token used by Certbot\ndns_cloudflare_api_token = {}",
        api_token
    );

    tokio::fs::write("/cloudflare.ini", config_content.as_bytes())
        .await
        .unwrap();
}
