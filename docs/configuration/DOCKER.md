# `docker.toml`

Optional. Defines per-host Docker daemon address pools and explicit network migration history.

```toml
schema_version = 1

[hosts.nexus]
bridge_ipv4_cidr = "172.17.0.0/16"
bridge_ipv6_cidr = "fdd7:d134:61e3:120::/64"
ipv4_pool_base = "10.200.0.0/12"
ipv4_pool_size = 24
ipv6_pool_base = "fdd7:d134:61e3:200::/56"
ipv6_pool_size = 64

[hosts.nexus.network_migrations.fuji]
previous_ipv6_cidr = "fdd7:d134:61e3:111::/64"
```

## `[hosts.<host>]`

| Key | Type | Required | Description |
|---|---|---:|---|
| `bridge_ipv4_cidr` | IPv4 CIDR | yes | Default Docker bridge IPv4 network. |
| `bridge_ipv6_cidr` | IPv6 CIDR | yes | Default Docker bridge IPv6 network. |
| `ipv4_pool_base` | IPv4 CIDR | yes | Base for Docker default address pools. |
| `ipv4_pool_size` | `u8` | yes | Prefix length of individual IPv4 pools. |
| `ipv6_pool_base` | IPv6 CIDR | yes | Base for IPv6 default address pools. |
| `ipv6_pool_size` | `u8` | yes | Prefix length of individual IPv6 pools. |
| `network_migrations` | map<table> | no | Explicit prior addressing for managed Docker networks. |

## `[hosts.<host>.network_migrations.<network>]`

| Key | Type | Required | Description |
|---|---|---:|---|
| `previous_ipv6_cidr` | IPv6 CIDR | no | Previous subnet used to generate guarded migration/rollback behavior. |

The host must exist in inventory. Render and inspect the Docker plan before applying because address-pool changes can require network recreation.
