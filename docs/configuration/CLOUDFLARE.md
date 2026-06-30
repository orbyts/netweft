# `cloudflare.toml`

Optional. Defines Cloudflare account/zone references and tunnel intent. Token values are never stored here; only environment-variable names are configured.

```toml
schema_version = 1

[providers.primary]
zone = "suhail.ink"
zone_id = "0123456789abcdef"
account_id = "fedcba9876543210"
tunnel_api_token_env = "CLOUDFLARE_TUNNEL_API_TOKEN"
dns_api_token_env = "CLOUDFLARE_DNS_API_TOKEN"

[tunnels.nexus-ingress]
provider = "primary"
connector_host = "nexus"
origin = "https://127.0.0.1:443"
origin_tls_verify = true
hostnames = ["dsm.suhail.ink"]
```

## `[providers.<provider>]`

| Key | Type | Required | Description |
|---|---|---:|---|
| `zone` | `string` | yes | DNS zone name managed by this provider entry. |
| `zone_id` | `string` | yes | Cloudflare zone identifier. |
| `account_id` | `string` | yes | Cloudflare account identifier owning tunnels. |
| `tunnel_api_token_env` | `string` | yes | Name of the environment variable containing the tunnel-management API token. |
| `dns_api_token_env` | `string` | yes | Name of the environment variable containing the DNS-management API token. |

Keep the two token purposes separate so each can be granted the narrowest permissions.

## `[tunnels.<tunnel>]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `provider` | `string` | yes | — | Provider identifier from `[providers]`. |
| `connector_host` | `string` | yes | — | Inventory host where `cloudflared` runs. |
| `origin` | `string` | yes | — | Origin URL reached by the connector, including scheme and port. |
| `origin_tls_verify` | `bool` | no | `true` | Whether `cloudflared` verifies the origin TLS certificate. Disable only with a deliberate local trust decision. |
| `hostnames` | `array<string>` | no | `[]` | Hostnames reconciled to the tunnel. |

The active location's `[external_ingress]` selects which provider and tunnel are used. The Cloudflare adapter may create/adopt remote tunnels, reconcile proxied DNS records, and emit a secret connector token. Treat rendered deployment environment files as secrets.
