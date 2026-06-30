# `proxmox-storages.toml`

Optional. Defines Proxmox storage entries, currently NFS-backed.

```toml
schema_version = 1

[storages.synology-backup]
host = "zion"
storage_id = "synology-backup"
provider = "nfs"
server_host = "ds1621plus"
export = "/volume1/proxmox"
mount_path = "/mnt/pve/synology-backup"
options = ["vers=4.1"]
content = ["backup", "iso", "vztmpl"]
prune_backups = "keep-last=7"
```

## `[storages.<storage>]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `host` | `string` | yes | — | Proxmox inventory host receiving the storage definition. |
| `storage_id` | `string` | yes | — | Proxmox storage identifier. |
| `provider` | enum | yes | — | Currently `nfs`. |
| `server_host` | `string` | yes | — | Inventory host exporting the storage. |
| `export` | `string` | yes | — | NFS export path. |
| `mount_path` | `string` | yes | — | Absolute Proxmox mount path. |
| `options` | `array<string>` | no | `[]` | Provider mount options. |
| `content` | `array<string>` | no | `[]` | Proxmox content types such as `backup`, `iso`, `vztmpl`, or `images`. |
| `prune_backups` | `string` | no | none | Proxmox prune-backups policy string. |
