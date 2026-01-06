pub struct DomainExpiration {
    pub domain: String,
    pub expiration: Option<String>,
    pub error: Option<String>,
}

pub async fn get_domains_with_expiration() -> Result<Vec<DomainExpiration>, String> {
    let domains = super::get_domains_list().await?;

    let mut result = Vec::new();

    for domain in domains {
        match super::get_cert_info(domain.as_str()).await {
            Ok(cert_info) => {
                result.push(DomainExpiration {
                    domain,
                    expiration: Some(cert_info.expires.to_rfc3339()),
                    error: None,
                });
            }
            Err(error) => {
                result.push(DomainExpiration {
                    domain,
                    expiration: None,
                    error: Some(error),
                });
            }
        }
    }

    Ok(result)
}
