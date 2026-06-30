# `locations/<name>.toml`

Attaches stable inventory to a physical site. This is where site-specific addressing, router capabilities, IPv6 mode, Tailscale policy, and external ingress selection live.

## Top level

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `schema_version` | `u32` | yes | — | Current value `1`. |
| `name` | `string` | yes | — | Location identifier. Normally matches the filename and `active_location`. |
| `description` | `string` | no | none | Human-readable description. |
| `router` | table | yes | — | Router capabilities. |
| `ipv6` | table | yes | — | Location IPv6 policy. |
| `segments` | map<table> | yes | — | LAN/VLAN segments. |
| `hosts` | map<table> | no | `{}` | Site attachment and addresses for inventory hosts. |
| `tailscale` | table | no | disabled | Overlay routing policy. |
| `external_ingress` | table | no | none | Cloudflare or disabled external ingress policy. |

## `[router]`

| Key | Type | Required | Description |
|---|---|---:|---|
| `kind` | `string` | yes | Router model/provider identifier. |
| `managed` | `bool` | yes | Whether Netweft may eventually consider it managed. Current adapters do not configure it directly. |
| `supports_vlans` | `bool` | yes | Declares VLAN capability. A warning is emitted if true but no VLAN segments exist. |

## `[ipv6]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `mode` | enum | yes | — | `disabled`, `router-advertised`, or `delegated`. |
| `prefix` | IPv6 CIDR | conditional | none | Current router-advertised or delegated prefix. Must be absent when disabled. |
| `subnet_prefix_length` | `u8` | no | `64` | Child subnet size for delegated allocation. Current allocation expects `/64`. |
| `stability` | enum | yes | — | `dynamic` or `stable`. |
| `publish_public_aaaa` | `bool` | no | `false` | Requests public AAAA publication. Dynamic router-advertised prefixes are not suitable for durable publication. |

### Mode semantics

- `disabled`: `prefix` must not be set.
- `router-advertised`: useful for current reachability and recursion ACLs, but not durable allocation.
- `delegated`: prefix is allocatable into child `/64` networks.

## `[segments.<segment>]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `kind` | enum | yes | — | `lan` or `vlan`. |
| `ipv4_cidr` | IPv4 CIDR | yes | — | Segment network. |
| `ipv4_gateway` | IPv4 | yes | — | Gateway, which must lie inside the CIDR. |
| `vlan_id` | `u16` | no | none | VLAN ID, normally for `kind = "vlan"`. |
| `public_ipv6_allocation` | `u32` | no | none | Delegated public IPv6 subnet allocation identifier. |
| `dns_clients` | `bool` | no | `true` | Makes eligible clients candidates for recursion ACLs. |
| `reverse_dns` | `bool` | no | `true` | Enables reverse-zone generation for the segment. |

## `[hosts.<host>.interfaces.<interface>]`

| Key | Type | Required | Description |
|---|---|---:|---|
| `segment` | `string` | yes | Segment identifier. Must exist. |
| `ipv4` | IPv4 | no | Static/reserved IPv4 identity at this location. Must lie inside the segment. |
| `ipv6_mode` | enum | no | `slaac`, `static`, or `disabled`. |
| `ula_interface_id` | `string` | no | Stable interface identifier used to derive a ULA where applicable. |

The host must exist in `inventory.toml`. A host network profile's `management_interface` must exist here.

## `[tailscale]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `enabled` | `bool` | no | `false` | Enables Tailscale planning. |
| `strategy` | enum | conditional | none | `subnet-router`, `ha-subnet-router`, or `direct-nodes`. |
| `primary_router` | `string` | conditional | none | Primary inventory host for router strategies. |
| `routers` | map<table> | no | `{}` | Per-router behavior. |

### `[tailscale.routers.<host>]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `enabled` | `bool` | no | `true` | Enables this router entry. |
| `snat_subnet_routes` | `bool` | no | `true` | Requests SNAT for advertised subnet routes. |
| `accept_routes` | `bool` | no | `false` | Accepts routes advertised by other nodes. |
| `exit_node` | `bool` | no | `false` | Enables exit-node behavior. |
| `advertise` | array<string> | no | `[]` | Selectors such as `segment:main` and `network:nexus-containers`. |

## `[external_ingress]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `mode` | enum | yes | — | `cloudflare-tunnel`, `cloudflare-direct`, or `disabled`. |
| `provider` | `string` | yes | — | Provider identifier from `cloudflare.toml`. |
| `tunnel` | `string` | conditional | none | Tunnel identifier for `cloudflare-tunnel`. |
| `origin_host` | `string` | no | none | Host that receives tunnel-origin traffic. |
| `publish_ipv4` | `bool` | no | `false` | Allows direct public IPv4 publication where supported. |
| `publish_ipv6` | `bool` | no | `false` | Allows direct public IPv6 publication where supported. |

`cloudflare-direct` is represented in the model but may be intentionally unsupported by the current adapter. Check the Cloudflare adapter documentation before using it.
