
``` yaml
services:
  certbot:
    image: ghcr.io/myjettools/certbot-api:0.1.0
    volumes:
    - /media/${user}/ext-ssd/etc_letsencrypt:/etc/letsencrypt
    - /media/${user}/ext-ssd/cloudflare.ini:/cloudflare.ini
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