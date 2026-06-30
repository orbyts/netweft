# Cloudflare ingress adapter

The `cloudflare` adapter resolves location-aware external ingress, reconciles a remotely managed Cloudflare Tunnel, reconciles proxied CNAME records, and renders the connector deployment for `cloudflared`.

## Inspect and render

```bash
netweft validate
netweft show cloudflare
netweft render cloudflare --tunnel nexus-ingress
```

When the active location already selects a tunnel, `--tunnel` is optional. Supplying a different tunnel is rejected.

## Output

```text
~/.local/share/netweft/generated/<location>/hosts/<connector>/cloudflare/<tunnel>/
├── action-plan.txt
├── tunnel-config.json
├── dns-plan.json
├── compose.yml
├── apply-cloudflare.sh
├── install.sh
├── verify.sh
└── rollback.sh
```

`apply-cloudflare.sh` later creates `deployment.env`; that file contains `TUNNEL_TOKEN` and `TUNNEL_ID` and is therefore a secret runtime artifact. It is not produced during rendering.

## Resolved behavior

The adapter validates that:

- the active location enables `cloudflare-tunnel`;
- the selected provider and tunnel exist;
- the tunnel uses the location-selected provider;
- the connector host exists and is enabled;
- at least one hostname is present;
- every hostname is a subdomain of the provider zone.

The adapter deliberately leaves the zone apex untouched. `cloudflare-direct` is represented in the schema but is not implemented.

## Secret boundary

Netweft stores only the environment-variable names for two tokens:

- a tunnel-management token;
- a DNS-management token.

The values must exist in the trusted shell that runs `apply-cloudflare.sh`. Netweft does not store them. The generated connector token in `deployment.env` must be transferred and stored with mode `0600`.

Deployment: [Cloudflare ingress deployment](../deployment/CLOUDFLARE.md)
