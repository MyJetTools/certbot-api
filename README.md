# certbot-api

HTTP + MCP front-end around `certbot` for issuing and renewing Let's Encrypt
certificates, plus a small Dioxus admin UI.

Two challenge types are supported:

- **DNS-01** (Cloudflare) — issues a single cert covering the apex **and** its
  wildcard (`example.com` + `*.example.com`). Endpoints under `/api/certbot` and
  `/api/certificates`; MCP tools `add_domain`, `renew_certificate`.
- **HTTP-01** (webroot) — issues a per-FQDN cert for the exact hostnames you
  list (no wildcards). Endpoints under `/api/http01`; MCP tools `init_http_01`,
  `reissue_http_01`.

## Tasks

Every long-running operation (issue / reissue / renew) runs as a background
**task** with an id, tracked in memory. Start endpoints return immediately with
a `task_id` (create-or-find by certificate name: if a task for that cert is
already running, its id is returned instead of starting a second one). Poll the
status endpoints, or list everything via `GET /api/tasks/v1/list` / MCP
`list_tasks`. Certbot runs are serialized process-wide and each is killed after
a 30-minute timeout.

## HTTP-01 setup

`init_http_01` runs `certbot certonly --webroot -w <webroot>` (default
`/var/www/acme`). Certbot writes the challenge token under
`<webroot>/.well-known/acme-challenge/<token>` and deletes it after validation —
**you never place files there by hand.** Let's Encrypt then fetches
`http://<domain>/.well-known/acme-challenge/<token>` **from the public internet,
over port 80 (plain HTTP)**, so that path must serve exactly that webroot.

What you must set up on the host:

1. **DNS** — the domain's `A`/`AAAA` record points to this host. (DNS-01 did not
   care where the request came from; HTTP-01 does.)
2. **Port 80 open to the whole internet** — Let's Encrypt validates from
   arbitrary IPs, so it cannot be firewalled to a whitelist.
3. **Route `/.well-known/acme-challenge/` to the shared webroot** on whatever
   owns `:80`. With my-reverse-proxy, share the webroot volume and add a location
   for that path **above** any http→https redirect (a 302 there fails
   validation).

Keep in mind:

- **Per hostname.** A multi-SAN cert validates every listed FQDN independently —
  each must resolve here and serve the challenge path, or the whole issuance
  fails.
- **Permanently, not just at issue.** The files come and go, but the `:80 →
  webroot` route must stay up — renewal (~every 60 days) re-validates.
- **Cloudflare-proxied domains** (orange cloud): the request reaches the origin,
  but "Always Use HTTPS" / WAF / cache can intercept
  `/.well-known/acme-challenge/*` — add a bypass for that path.

Sanity check before issuing, from an outside machine:

```bash
curl http://<domain>/.well-known/acme-challenge/test
# reaches the container (a 404 from the service is fine — the route works)
# vs. connection refused / timeout / a 301 to https — not wired up yet
```

## Admin UI

A client-side Dioxus app (`ui/`) served as static files from `./wwwroot` by the
same server (same-origin API calls). It shows the served domains with their
expiry and the current/recent tasks.

Rebuild it after changing anything under `ui/`, then commit the updated
`wwwroot/`:

```bash
./build-ui.sh        # runs dx build --release --web and copies into wwwroot/
```

`wwwroot/` is committed, so CI needs no extra step — it just `cargo build`s and
`docker build`s, and the Dockerfile copies `wwwroot` into the image.

## Deployment

```yaml
services:
  certbot:
    image: ghcr.io/myjettools/certbot-api:0.1.0
    volumes:
    - /media/${user}/ext-ssd/etc_letsencrypt:/etc/letsencrypt
    - /media/${user}/ext-ssd/cloudflare.ini:/cloudflare.ini
    # HTTP-01 webroot, shared with whatever serves :80 (e.g. the reverse proxy)
    - /media/${user}/ext-ssd/acme_webroot:/var/www/acme
    ports:
    - "8005:8000"
    logging:
      options:
        max-size: "512Kb"
        max-file: "1"

    networks:
    - docker_net

networks:
  docker_net:
    external: true
```

The admin UI is served on the same port (`/`); the API is under `/api`, Swagger
under `/swagger`, MCP under `/mcp`.
