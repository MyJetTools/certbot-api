// Certbot stores certificates under the apex name regardless of whether the
// original request was wildcard: "*.example.com" and "example.com" both live
// in /etc/letsencrypt/live/example.com. Every entry point that accepts a
// domain must normalize through here so both forms behave identically.
pub fn normalize_cert_name(domain: &str) -> String {
    let domain = domain.trim();
    domain.strip_prefix("*.").unwrap_or(domain).to_string()
}
