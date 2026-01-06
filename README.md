

Source docker configuration is:
``` yaml
services:
  certbot:
    image: certbot/dns-cloudflare
    volumes:
    - /media/user/ext-ssd/etc_letsencrypt:/etc/letsencrypt
    - /media/user/ext-ssd/cloudflare.ini:/cloudflare.ini
```