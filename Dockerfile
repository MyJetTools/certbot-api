FROM certbot/dns-cloudflare

COPY ./target/release/certbot-api /app/certbot-api
WORKDIR /app
ENTRYPOINT ["/app/certbot-api"]
