# Runtime stage
FROM ubuntu:22.04

# Install certbot with Cloudflare DNS plugin support
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    certbot \
    python3-certbot-dns-cloudflare \
    ca-certificates \
    libc6 \
    && rm -rf /var/lib/apt/lists/*

COPY ./target/release/certbot-api /app/certbot-api
WORKDIR /app
ENTRYPOINT ["/app/certbot-api"]
