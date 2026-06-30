# `inventory.toml`

Defines stable domains and machine identities. Site-specific addresses belong in a location file, not here.

## Example

```toml
schema_version = 1

[domains]
primary = "suhail.ink"
additional = ["suhail.photos", "suhail.art", "suhail.life"]

[hosts.nexus]
kind = "physical"
roles = ["infrastructure", "dns", "proxy", "docker", "tailscale"]
runtime_root = "/var/lib/suhail/services/nexus"
ssh_user = "suhail"
enabled = true
```

## `[domains]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `primary` | `string` | yes | — | Primary DNS suffix used by the deployment. |
| `additional` | `array<string>` | no | `[]` | Additional managed domains. |

## `[hosts.<host>]`

`<host>` is the stable host identifier referenced by locations, services, guests, routes, mounts, and adapters.

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `kind` | enum | yes | — | One of `physical`, `vm`, `laptop`, `workstation`, `nas`. |
| `roles` | `array<string>` | no | `[]` | Descriptive capabilities. Roles are not currently a permission system. |
| `parent` | `string` | no | none | Parent host identifier, typically for a VM. |
| `runtime_root` | `string` | no | none | Host-local root for mutable service state. Prefer an absolute path outside Git. |
| `ssh_user` | `string` | no | none | Default SSH user associated with this host. |
| `network` | table | no | none | Stable Proxmox/ifupdown2 host network topology. |
| `os_network` | table | no | none | Stable operating-system network policy for Netplan-managed hosts. |
| `enabled` | `bool` | no | `true` | Whether the host participates in planning. |

## `[hosts.<host>.network]`

Used by the Proxmox host-network adapter.

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `provider` | enum | yes | — | Currently `proxmox-ifupdown2`. |
| `management_interface` | `string` | yes | — | Name of the location interface carrying host management addressing. |
| `preserve_includes` | `bool` | no | `false` | Preserves existing include/source directives when rendering the host network file. |
| `links` | array<table> | no | `[]` | Physical or logical link declarations. |
| `bridges` | array<table> | no | `[]` | Bridge declarations. |

### `[[hosts.<host>.network.links]]`

| Key | Type | Required | Values | Description |
|---|---|---:|---|---|
| `name` | `string` | yes | — | Interface name. Must be non-empty and unique within the host profile. |
| `kind` | enum | yes | `ethernet`, `wifi` | Link kind. |

### `[[hosts.<host>.network.bridges]]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `name` | `string` | yes | — | Bridge name, such as `vmbr0`. |
| `ports` | `array<string>` | no | `[]` | Member interfaces. |
| `location_interface` | `string` | no | none | Location interface whose addressing should be applied to this bridge. |
| `vlan_aware` | `bool` | no | `false` | Enables VLAN-aware bridge behavior. |
| `allowed_vlans` | `string` | no | none | Provider-specific VLAN range/list, such as `2-4094`. |
| `stp` | `bool` | no | `false` | Enables spanning tree. |
| `forward_delay` | `u32` | no | `0` | Bridge forwarding delay. |
| `comment` | `string` | no | none | Human-readable comment in generated configuration. |

## `[hosts.<host>.os_network]`

Used by the Netplan adapter.

| Key | Type | Required | Values | Description |
|---|---|---:|---|---|
| `provider` | enum | yes | `netplan` | OS network configuration provider. |
| `renderer` | enum | yes | `networkd` | Netplan renderer. |
| `interface` | `string` | yes | — | OS interface name to configure. |
| `ipv4_mode` | enum | yes | `dhcp` | IPv4 configuration mode. |
| `ipv6_mode` | enum | yes | `disabled`, `router-advertised` | IPv6 behavior. |

The OS profile is stable host policy. Actual expected addresses are resolved from the selected location.
