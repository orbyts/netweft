# `proxmox-sdn.toml`

Optional. Defines Proxmox SDN zones, VNets, and IPv4 DHCP subnets.

```toml
schema_version = 1

[zones.lab]
host = "zion"
kind = "simple"
ipam = "pve"
dhcp = "dnsmasq"

[vnets.services]
zone = "lab"
alias = "Service network"

[[vnets.services.subnets]]
cidr = "10.74.0.0/16"
gateway = "10.74.0.1"
dhcp_start = "10.74.10.1"
dhcp_end = "10.74.10.254"
snat = true
```

## `[zones.<zone>]`

| Key | Type | Required | Values | Description |
|---|---|---:|---|---|
| `host` | `string` | yes | — | Proxmox inventory host owning the zone. |
| `kind` | enum | yes | `simple` | Proxmox SDN zone type. |
| `ipam` | enum | yes | `pve` | IPAM provider. |
| `dhcp` | enum | yes | `dnsmasq` | DHCP provider. |

## `[vnets.<vnet>]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `zone` | `string` | yes | — | Zone identifier; must exist. |
| `alias` | `string` | no | none | Human-readable alias. |
| `subnets` | array<table> | no | `[]` | IPv4 subnets attached to the VNet. |

## `[[vnets.<vnet>.subnets]]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `cidr` | IPv4 CIDR | yes | — | Subnet network. |
| `gateway` | IPv4 | yes | — | Gateway inside the CIDR. |
| `dhcp_start` | IPv4 | yes | — | First DHCP address. |
| `dhcp_end` | IPv4 | yes | — | Last DHCP address. |
| `snat` | `bool` | no | `false` | Enables source NAT for the subnet. |
