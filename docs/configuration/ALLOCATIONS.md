# `allocations.toml`

Provides durable ULA allocation IDs. These values are infrastructure identity and should not be casually changed after deployment.

```toml
schema_version = 1

[ula]
prefix = "fdd7:d134:61e3::/48"

[ula.segments]
main = 256

[ula.networks]
nexus-containers = 289
vortex-containers = 321
```

## `[ula]`

| Key | Type | Required | Description |
|---|---|---:|---|
| `prefix` | IPv6 CIDR | yes | ULA allocation root, typically a `/48`. |
| `segments` | map<u16> | no | Stable subnet IDs for location segments. |
| `networks` | map<u16> | no | Stable subnet IDs for logical networks. |

Map values are decimal integers but are used as IPv6 subnet identifiers. For example, decimal `289` is hexadecimal `0x121`.

Every key in `[ula.networks]` must match a `network.allocation_key` from `networks.toml`.
