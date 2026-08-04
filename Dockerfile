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
# The Dioxus admin UI, built by ./build-ui.sh into ./wwwroot (committed) and
# served as static files. The server resolves ./wwwroot relative to its CWD.
COPY ./wwwroot /app/wwwroot
WORKDIR /app
ENTRYPOINT ["/app/certbot-api"]
