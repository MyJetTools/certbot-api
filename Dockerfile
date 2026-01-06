FROM certbot/dns-cloudflare

COPY ./target/release/certbot-api ./target/release/certbot-api
ENTRYPOINT ["./target/release/certbot-api"]
