# Configuration option index

This page is the fast lookup companion to the detailed configuration chapters. Keys are grouped by file and table path. Follow the linked page for examples, validation rules, and generated effects.

All modelled structures reject unknown keys. A typo is an error, not an ignored setting.

## `netweft.toml`

| Path | Key | Type | Default | Meaning |
|---|---|---|---|---|
| top level | `schema_version` | integer | required | Configuration schema; currently `1`. |
| top level | `active_location` | string | required | Location filename without `.toml`. |
| `paths` | `generated_root` | path | XDG data root | Generated artifacts root. |
| `paths` | `state_root` | path | XDG state root | Persistent Netweft state. |
| `paths` | `cache_root` | path | XDG cache root | Rebuildable cache. |
| `render` | `atomic` | bool | `true` | Request atomic replacement where supported. |
| `render` | `stable_order` | bool | `true` | Deterministic ordering. |
| `render` | `generated_headers` | bool | `true` | Add generated-file headers where supported. |
| `validation` | `warn_dynamic_ipv6` | bool | `false` | Warn about non-durable RA prefixes. |
| `validation` | `warn_dropbox_runtime` | bool | `false` | Warn about mutable runtime state in sync storage. |
| `validation` | `warn_latest_images` | bool | `false` | Warn about floating image tags. |
| `validation` | `deny_warnings` | bool | `false` | Promote warnings to errors. |

See [NETWEFT.md](NETWEFT.md).

## `inventory.toml`

| Path | Key | Type/default | Meaning |
|---|---|---|---|
| `domains` | `primary` | string, required | Primary managed DNS suffix. |
| `domains` | `additional` | string array, `[]` | Other managed domains. |
| `hosts.<id>` | `kind` | enum, required | `physical`, `vm`, `laptop`, `workstation`, or `nas`. |
| `hosts.<id>` | `roles` | string array, `[]` | Descriptive capabilities. |
| `hosts.<id>` | `parent` | host id, optional | Parent hypervisor/host. |
| `hosts.<id>` | `runtime_root` | path, optional | Mutable runtime state root. |
| `hosts.<id>` | `ssh_user` | string, optional | Default SSH user. |
| `hosts.<id>` | `enabled` | bool, `true` | Include host in planning. |
| `hosts.<id>.network` | `provider` | `proxmox-ifupdown2` | Host-network renderer. |
| `hosts.<id>.network` | `management_interface` | interface id, required | Location attachment that supplies management addressing. |
| `hosts.<id>.network` | `preserve_includes` | bool, `false` | Preserve include/source directives. |
| `network.links[]` | `name` | string, required | OS interface name. |
| `network.links[]` | `kind` | `ethernet` or `wifi` | Link type. |
| `network.bridges[]` | `name` | string, required | Bridge name. |
| `network.bridges[]` | `ports` | string array, `[]` | Bridge member links. |
| `network.bridges[]` | `location_interface` | interface id, optional | Location attachment applied to bridge. |
| `network.bridges[]` | `vlan_aware` | bool, `false` | Enable tagged VLAN handling. |
| `network.bridges[]` | `allowed_vlans` | string, optional | Provider VLAN range/list. |
| `network.bridges[]` | `stp` | bool, `false` | Spanning Tree Protocol. |
| `network.bridges[]` | `forward_delay` | integer, `0` | Bridge forwarding delay. |
| `network.bridges[]` | `comment` | string, optional | Generated comment. |
| `hosts.<id>.os_network` | `provider` | `netplan` | OS network renderer. |
| `os_network` | `renderer` | `networkd` | Netplan backend. |
| `os_network` | `interface` | string, required | OS interface to configure. |
| `os_network` | `ipv4_mode` | `dhcp` | IPv4 mode. |
| `os_network` | `ipv6_mode` | `disabled` or `router-advertised` | IPv6 behavior. |

See [INVENTORY.md](INVENTORY.md).

## `locations/<name>.toml`

| Path | Key | Type/default | Meaning |
|---|---|---|---|
| top level | `name` | string, required | Location id. |
| top level | `description` | string, optional | Human-readable site description. |
| `router` | `kind` | string, required | Router model/provider label. |
| `router` | `managed` | bool, required | Whether router is considered managed. |
| `router` | `supports_vlans` | bool, required | Router VLAN capability. |
| `ipv6` | `mode` | enum, required | `disabled`, `router-advertised`, or `delegated`. |
| `ipv6` | `prefix` | IPv6 CIDR, conditional | Current RA or delegated prefix. |
| `ipv6` | `subnet_prefix_length` | integer, `64` | Child subnet size. |
| `ipv6` | `stability` | `dynamic` or `stable` | Prefix durability. |
| `ipv6` | `publish_public_aaaa` | bool, `false` | Allow public AAAA publication. |
| `segments.<id>` | `kind` | `lan` or `vlan` | Segment type. |
| `segments.<id>` | `ipv4_cidr` | IPv4 CIDR, required | Segment network. |
| `segments.<id>` | `ipv4_gateway` | IPv4, required | Segment gateway. |
| `segments.<id>` | `vlan_id` | integer, optional | VLAN tag. |
| `segments.<id>` | `public_ipv6_allocation` | integer, optional | Delegated subnet id. |
| `segments.<id>` | `dns_clients` | bool, `true` | Include in recursion candidates. |
| `segments.<id>` | `reverse_dns` | bool, `true` | Generate reverse DNS. |
| `hosts.<host>.interfaces.<id>` | `segment` | segment id, required | Attached segment. |
| same | `ipv4` | IPv4, optional | Stable site address. |
| same | `ipv6_mode` | `slaac`, `static`, `disabled` | Interface IPv6 policy. |
| same | `ula_interface_id` | string, optional | Stable ULA host/interface id. |
| `tailscale` | `enabled` | bool, `false` | Enable Tailscale planning. |
| `tailscale` | `strategy` | enum, conditional | `subnet-router`, `ha-subnet-router`, `direct-nodes`. |
| `tailscale` | `primary_router` | host id, conditional | Primary route advertiser. |
| `tailscale.routers.<host>` | `enabled` | bool, `true` | Enable router entry. |
| same | `snat_subnet_routes` | bool, `true` | SNAT advertised routes. |
| same | `accept_routes` | bool, `false` | Accept peer routes. |
| same | `exit_node` | bool, `false` | Advertise exit-node behavior. |
| same | `advertise` | selector array, `[]` | `segment:<id>` and `network:<id>` selectors. |
| `external_ingress` | `mode` | enum, required | `cloudflare-tunnel`, `cloudflare-direct`, `disabled`. |
| same | `provider` | provider id, conditional | Cloudflare provider. |
| same | `tunnel` | tunnel id, conditional | Named tunnel. |
| same | `origin_host` | host id, optional | Origin connector host override. |
| same | `publish_ipv4` | bool, `false` | Direct IPv4 publication. |
| same | `publish_ipv6` | bool, `false` | Direct IPv6 publication. |

See [LOCATIONS.md](LOCATIONS.md).

## `networks.toml`

| Path | Key | Type/default | Meaning |
|---|---|---|---|
| `networks.<id>` | `kind` | `lan` or `docker` | Logical network type. |
| same | `owner` | host id, optional | Owning host. |
| same | `docker_name` | string, optional | Runtime Docker network name. |
| same | `ipv4_cidr` | IPv4 CIDR, optional | Network space. |
| same | `ipv4_gateway` | IPv4, optional | Gateway inside CIDR. |
| same | `allocation_key` | string, required | ULA allocation lookup key. |
| same | `preferred_display_id` | integer, optional | Stable human-friendly id. |
| same | `ula_enabled` | bool, `false` | Derive ULA subnet. |
| same | `reverse_dns` | bool, `false` | Generate reverse zone. |
| same | `dns_clients` | bool, `true` | Include in recursion candidates. |
| `routing` | `mode` | enum, required | `direct`, `via-host`, `host-private`. |
| `routing` | `from` | host id, conditional | Reachability source. |
| `routing` | `via` | host id, conditional | Next-hop host. |

See [NETWORKS.md](NETWORKS.md).

## `services.toml`

| Path | Key | Type/default | Meaning |
|---|---|---|---|
| `certificates.<id>` | `domains` | string array, `[]` | Covered names. |
| same | `certificate_path` | absolute path, required | Runtime certificate/full chain. |
| same | `private_key_path` | absolute path, required | Runtime private key. |
| `services.<id>` | `kind` | enum, required | `dns`, `reverse-proxy`, `development-container`, `web`, `database`, `generic`. |
| same | `host` | host id, required | Service placement. |
| same | `network` | network id, required | Service network. |
| same | `enabled` | bool, `true` | Include service in plans. |
| `address` | `ipv4` | IPv4, optional | Service address inside network CIDR. |
| `address` | `ipv6_interface_id` | string, optional | ULA interface id. |
| `ports[]` | `host` | port, required | Host-side port. |
| `ports[]` | `container` | port, required | Internal port. |
| `ports[]` | `protocol` | `tcp` or `udp` | Transport. |
| `ingress` | `mode` | `host-port` | Host ingress model. |
| `ingress` | `interface` | interface id, required | Location interface. |
| `ssh` | `user` | string, required | SSH user. |
| `ssh` | `host_port` | port, required | Exposed host port. |
| `ssh` | `container_port` | port, required | Internal SSH port. |
| `ssh` | `route` | string, required | SSH route selector. |
| `web` | `domain` | domain, required | Public/canonical hostname. |
| `web` | `access` | `reverse-proxy` or `direct` | Access model. |
| `web` | `proxy` | service id, conditional | Reverse-proxy service. |
| `web` | `scheme` | `http` or `https` | Upstream scheme. |
| `web` | `container_port` | port, optional | Container upstream port. |
| `web` | `upstream_host` | string, optional | Explicit upstream host/IP. |
| `web` | `upstream_port` | port, optional | Explicit upstream port. |
| `web` | `tls` | bool, `false` | Serve TLS at proxy. |
| `web` | `certificate` | certificate id, conditional | TLS certificate reference. |
| `web` | `force_https` | bool, `false` | Redirect HTTP to HTTPS. |
| `web` | `websocket` | bool, `false` | Enable WebSocket proxy headers. |

See [SERVICES.md](SERVICES.md).

## Optional subsystem files

The detailed pages are the canonical references for optional adapters:

- [DNS.md](DNS.md) — BIND settings, SOA, recursion, forwarders, zones, and records.
- [ALLOCATIONS.md](ALLOCATIONS.md) — durable ULA global and network ids.
- [DOCKER.md](DOCKER.md) — daemon settings and network migration policy.
- [GUESTS.md](GUESTS.md) — Proxmox VM/LXC reconciliation.
- [PROXMOX-SDN.md](PROXMOX-SDN.md) — zones, VNets, subnets, DHCP, and SNAT.
- [PROXMOX-STORAGES.md](PROXMOX-STORAGES.md) — NFS and other Proxmox storage definitions.
- [MOUNTS.md](MOUNTS.md) — systemd network mounts.
- [NAS-PERMISSIONS.md](NAS-PERMISSIONS.md) — DSM NFS permission plans.
- [SSH.md](SSH.md) — client identities and targets.
- [CLOUDFLARE.md](CLOUDFLARE.md) — providers, token-env references, tunnels, and routes.

## How to verify a key

Use three checks before deployment:

```bash
netweft validate
netweft adapters list
find ~/.local/share/netweft/generated -type f | sort
```

`validate` proves that the model is internally consistent. It does not prove that a remote host has the expected NIC names, files, credentials, or provider state. Inspect those facts separately before applying a render.
