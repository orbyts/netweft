# Configuration

```text
~/.config/netweft/
├── netweft.toml
├── inventory.toml
├── networks.toml
├── services.toml
├── dns.toml
├── allocations.toml
└── locations/
```

| File | Responsibility |
|---|---|
| `netweft.toml` | Global settings and active location |
| `inventory.toml` | Stable host identities |
| `networks.toml` | Stable logical networks |
| `services.toml` | Services and placement |
| `dns.toml` | Zones, records, recursion, forwarding |
| `allocations.toml` | Durable ULA allocation IDs |
| `locations/*.toml` | Site-specific addressing and routing |

Dynamic ISP prefixes are not durable host identity. Secrets remain outside configuration.


## Cloudflare ingress

Cloudflare configuration is split between the provider/tunnel inventory and the active location.

`cloudflare.toml` stores identifiers and the names of environment variables that supply API tokens. Token values never belong in Netweft configuration.

```toml
schema_version = 1

[providers.suhail-ink]
zone = "suhail.ink"
zone_id = "<cloudflare-zone-id>"
account_id = "<cloudflare-account-id>"
tunnel_api_token_env = "CLOUDFLARE_TUNNEL_API_TOKEN"
dns_api_token_env = "CLOUDFLARE_DNS_API_TOKEN"

[tunnels.nexus-ingress]
provider = "suhail-ink"
connector_host = "nexus"
origin = "https://127.0.0.1:443"
origin_tls_verify = false
hostnames = ["dsm.suhail.ink"]
```

The active location selects the ingress mode and tunnel:

```toml
[external_ingress]
mode = "cloudflare-tunnel"
provider = "suhail-ink"
tunnel = "nexus-ingress"
publish_ipv4 = false
publish_ipv6 = false
```

Supported modes are `disabled`, `cloudflare-tunnel`, and `cloudflare-direct`. Direct-origin mode is modeled but not implemented. Tunnel hostnames must be subdomains of the configured zone; the zone apex is intentionally excluded.

## SSH client profiles

SSH configuration resolves aliases against location-aware host, guest, or service addresses. A target must select exactly one source kind. Identity paths are client-specific and remain references to existing private-key files.
