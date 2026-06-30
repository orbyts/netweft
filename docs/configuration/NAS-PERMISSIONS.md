# `nas-permissions.toml`

Optional. Declares intended NFS client permissions on a NAS. The current provider maps intent to Synology DSM terminology.

```toml
schema_version = 1

[permissions.vortex-media]
nas = "ds1621plus"
provider = "synology-nfs"
share = "media"
client_host = "vortex"
access = "read-write"
squash = "map-root-to-admin"
security = "sys"
```

## `[permissions.<permission>]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `nas` | `string` | yes | — | NAS inventory host. |
| `provider` | enum | yes | — | Currently `synology-nfs`. |
| `share` | `string` | yes | — | Non-empty NAS share name. |
| `client_host` | `string` | yes | — | Inventory host receiving access. |
| `access` | enum | yes | — | `read-only` or `read-write`. |
| `squash` | enum | yes | — | `none`, `map-root-to-admin`, `map-all-to-admin`, or `map-all-to-guest`. |
| `security` | `string` | no | `sys` | NFS security flavor shown/applied by the provider workflow. Must be non-empty. |

This adapter may produce instructions rather than automatically editing DSM, depending on the current implementation.
