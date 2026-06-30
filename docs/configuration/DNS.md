# `dns.toml`

Defines DNS implementation selection, SOA timing, recursion policy, forwarding, zones, and records.

## `[dns]`

| Key | Type | Required | Description |
|---|---|---:|---|
| `provider` | `string` | yes | DNS provider identifier, currently normally `bind9`. |
| `service` | `string` | yes | Service identifier for the DNS server. Must exist in `services.toml`. |
| `default_ttl` | `u32` | yes | Default positive-record TTL in seconds. |
| `negative_ttl` | `u32` | yes | Negative cache TTL in seconds. |
| `soa` | table | yes | SOA values. |
| `recursion` | table | yes | Recursion ACL derivation policy. |
| `forwarders` | table | yes | Upstream recursive resolvers. |

## `[dns.soa]`

| Key | Type | Required | Description |
|---|---|---:|---|
| `primary_nameserver` | `string` | yes | MNAME, normally fully qualified with trailing dot. |
| `responsible_mailbox` | `string` | yes | RNAME encoded as a DNS name, normally with trailing dot. |
| `refresh` | `u32` | yes | Secondary refresh interval. |
| `retry` | `u32` | yes | Retry interval. |
| `expire` | `u32` | yes | Zone expiration interval. |

## `[dns.recursion]`

All keys default to `true`.

| Key | Description |
|---|---|
| `enabled` | Enables recursive service and ACL derivation. |
| `include_location_segments` | Includes eligible location segments with `dns_clients = true`. |
| `include_tailscale` | Includes applicable Tailscale address space/routes. |
| `include_ula` | Includes Netweft ULA space. |
| `include_docker_networks` | Includes reachable logical Docker networks with `dns_clients = true`. |

A network is not included merely because it exists; routing policy must make it reachable from the DNS host.

## `[dns.forwarders]`

| Key | Type | Default | Description |
|---|---|---|---|
| `ipv4` | array<IPv4> | `[]` | IPv4 upstream resolvers. |
| `ipv6` | array<IPv6> | `[]` | IPv6 upstream resolvers. |

## `[[zones]]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `name` | `string` | yes | Zone apex without a required trailing dot. |
| `visibility` | enum | yes | `internal`, `public`, or `both`. |
| `authoritative` | `bool` | no | `false` | Whether Netweft renders authoritative zone data. |

## `[[records]]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `name` | `string` | yes | Fully qualified record owner name. |
| `kind` | enum | yes | `host`, `service`, `proxy`, `cname`, or `segment-gateway`. |
| `target` | `string` | conditional | Identifier or name interpreted according to `kind`. |
| `interface` | `string` | conditional | Host location interface used by `host` records. |
| `address_scope` | enum | conditional | `container` or `ingress` for service address selection. |
| `families` | array<enum> | no | `["ipv4"]` | Address families: `ipv4`, `ipv6`. |
| `reverse` | `bool` | no | `false` | Requests PTR generation when the record resolves to an address. |

### Record-kind behavior

- `host`: `target` names an inventory host; `interface` selects its location interface.
- `service`: `target` names a service; `address_scope` chooses container or ingress addressing.
- `proxy`: resolves through the selected reverse-proxy ingress.
- `cname`: `target` is the canonical DNS name.
- `segment-gateway`: `target` names a location segment.

Only one preferred PTR should normally be generated per address.
