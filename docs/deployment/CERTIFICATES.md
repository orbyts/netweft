# Certificates

Netweft references certificate paths but does not issue or renew certificates.

Current Nexus manager: Certbot with Cloudflare DNS validation.

Runtime root:

```text
/var/lib/suhail/services/nexus/certbot
```

Credential file, never committed:

```text
/var/lib/suhail/services/nexus/certbot/secrets/cloudflare.ini
```

Issue:

```bash
cd "$DOCKER/services/nexus"
sudo ./certbot/scripts/issue-suhail-ink.sh
```

Renew and deploy:

```bash
sudo ./certbot/scripts/renew-and-deploy.sh
```

Automation:

```text
netweft-certbot-renew.service
netweft-certbot-renew.timer
```
