# `guests.toml`

Optional. Defines Proxmox VM and LXC identity, addressing, startup behavior, and VM-only attachments.

```toml
schema_version = 1

[guests.vortex]
kind = "vm"
host = "zion"
vmid = 201
mac = "02:00:00:00:02:01"
interface = "lan"
bridge = "vmbr0"
ipv4 = "10.214.90.50"
address_mode = "reserved-dhcp"
onboot = true
startup = "order=20,up=30"
firewall = true

[[guests.vortex.pci_devices]]
slot = 0
device = "0000:01:00.0"

[[guests.vortex.virtiofs]]
slot = 0
directory = "/srv/shared"
expose_acl = false
```

## `[guests.<guest>]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `kind` | enum | yes | — | `vm` or `lxc`. |
| `host` | `string` | yes | — | Parent Proxmox inventory host. |
| `vmid` | `u32` | yes | — | VMID, unique on the parent host. |
| `mac` | `string` | yes | — | Globally unique guest MAC address. |
| `interface` | `string` | yes | — | Location interface/segment attachment used for address resolution. |
| `bridge` | `string` | no | `vmbr0` | Proxmox bridge or VNet name. |
| `ipv4` | IPv4 | yes | — | Unique intended guest IPv4 address. |
| `address_mode` | enum | yes | — | `reserved-dhcp` or `static`. |
| `onboot` | `bool` | no | `false` | Starts guest automatically with the host. |
| `startup` | `string` | no | none | Proxmox startup policy string. |
| `firewall` | `bool` | no | `false` | Enables the guest NIC firewall flag. |
| `pci_devices` | array<table> | no | `[]` | PCI passthrough attachments; VM-only. |
| `virtiofs` | array<table> | no | `[]` | VirtioFS attachments; VM-only. |

### PCI attachment

| Key | Type | Required | Description |
|---|---|---:|---|
| `slot` | `u8` | yes | Unique attachment slot in this guest. |
| `device` | `string` | yes | Non-empty PCI device identifier. A device cannot be claimed by multiple guests. |

### VirtioFS attachment

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `slot` | `u8` | yes | — | Unique VirtioFS slot. |
| `directory` | `string` | yes | — | Non-empty host directory. |
| `expose_acl` | `bool` | no | `false` | Exposes ACL behavior where supported. |
