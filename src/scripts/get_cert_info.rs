use my_tls::cert_info::CertificateInfo;

pub async fn get_cert_info(domain: &str) -> Result<CertificateInfo, String> {
    let cert = super::get_fullchain(domain).await?;
    let private_key = super::get_private_key(domain).await?;

    let cert = my_tls::cert_info::SslCertificate::new(private_key.into_bytes(), cert.into_bytes())?;

    Ok(cert.get_cert_info())
}
