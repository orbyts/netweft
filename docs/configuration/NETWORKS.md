# `networks.toml`

Defines stable logical networks independently of any physical location.

## Example

```toml
schema_version = 1

[networks.nexus-containers]
kind = "docker"
owner = "nexus"
docker_name = "fuji"
ipv4_cidr = "10.78.0.0/16"
ipv4_gateway = "10.78.0.1"
allocation_key = "nexus-containers"
preferred_display_id = 121
ula_enabled = true
reverse_dns = true
dns_clients = true

[networks.nexus-containers.routing]
from = "nexus"
mode = "direct"
```

## `[networks.<network>]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `kind` | enum | yes | — | `lan` or `docker`. |
| `owner` | `string` | no | none | Host that owns the network. Must reference an inventory host when present. |
| `docker_name` | `string` | no | none | Runtime Docker network name. |
| `ipv4_cidr` | IPv4 CIDR | no | none | IPv4 address space. |
| `ipv4_gateway` | IPv4 | no | none | Gateway; when both are present, it must lie inside `ipv4_cidr`. |
| `allocation_key` | `string` | yes | — | Stable key used to match a ULA allocation in `allocations.toml`. |
| `preferred_display_id` | `u16` | no | none | Human-friendly stable numeric identifier used for display or derivation. |
| `ula_enabled` | `bool` | no | `false` | Whether a ULA subnet should be derived for this network. |
| `reverse_dns` | `bool` | no | `false` | Whether reverse DNS should be generated for the network. |
| `dns_clients` | `bool` | no | `true` | Whether reachable clients on this network may be considered for DNS recursion ACLs. |
| `routing` | table | no | none | Reachability policy from an infrastructure host. |

## `[networks.<network>.routing]`

| Key | Type | Required | Description |
|---|---|---:|---|
| `mode` | enum | yes | `direct`, `via-host`, or `host-private`. |
| `from` | `string` | conditional | Host from which the route is considered reachable. |
| `via` | `string` | conditional | Gateway host for `via-host`. |

### Routing rules

- `direct`: `via` must be absent. `from` may identify the directly connected host.
- `via-host`: `via` is required and must reference an inventory host.
- `host-private`: both `from` and `via` must be absent.
- Tailscale cannot advertise a `host-private` network.
- An advertised logical network must be routed from the advertising router.
