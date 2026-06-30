# `mounts.toml`

Optional. Defines network filesystem mounts consumed by hosts and rendered as systemd mount units.

```toml
schema_version = 1

[mounts.media]
host = "vortex"
provider = "nfs"
server_host = "ds1621plus"
export = "/volume1/media"
mount_path = "/mnt/media"
options = ["nfsvers=4.1", "hard", "_netdev"]
required_by = ["docker.service"]
```

## `[mounts.<mount>]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `host` | `string` | yes | — | Consumer inventory host. |
| `provider` | enum | yes | — | Currently `nfs`. |
| `server_host` | `string` | yes | — | Inventory host exporting the filesystem. |
| `export` | `string` | yes | — | Server-side export path. |
| `mount_path` | `string` | yes | — | Absolute local path. Relative paths are rejected. |
| `options` | `array<string>` | no | `[]` | Mount options passed to the generated unit. |
| `required_by` | `array<string>` | no | `[]` | systemd units that require/order after the mount. |
