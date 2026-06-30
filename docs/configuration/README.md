# Configuration reference

Start here:

- [Build a portable homelab: complete example](EXAMPLE-TOPOLOGY.md) — a file-by-file setup guide using a real topology.
- [Configuration option index](OPTION-INDEX.md) — fast lookup for keys, types, defaults, and meaning.

This is the canonical reference for Netweft TOML configuration. It is intended to be sufficient for authoring configuration without reading Rust source.

## Conventions

Every configuration structure uses `deny_unknown_fields`. A misspelled or unsupported key is an error rather than being ignored.

Unless stated otherwise:

- `schema_version` is required and must be `1`;
- map keys such as `[hosts.nexus]` or `[services.bind9]` are user-defined stable identifiers;
- `Option<T>` fields are optional and have no implicit value;
- fields marked with a default may be omitted;
- paths expected by deployment adapters should normally be absolute;
- names referenced from another file must exist and pass validation.

## Load order and dependencies

```text
netweft.toml
    selects a location

inventory.toml
    defines stable hosts and domains

networks.toml
    defines logical networks

locations/<name>.toml
    attaches hosts and networks to a site

services.toml
    places services on hosts and networks

dns.toml
    references hosts, services, segments, and proxies

allocations.toml
    supplies durable ULA allocation IDs
```

Optional subsystems:

- [docker.toml](DOCKER.md)
- [ssh.toml](SSH.md)
- [cloudflare.toml](CLOUDFLARE.md)
- [guests.toml](GUESTS.md)
- [mounts.toml](MOUNTS.md)
- [nas-permissions.toml](NAS-PERMISSIONS.md)
- [proxmox-storages.toml](PROXMOX-STORAGES.md)
- [proxmox-sdn.toml](PROXMOX-SDN.md)

## Core files

- [netweft.toml](NETWEFT.md)
- [inventory.toml](INVENTORY.md)
- [networks.toml](NETWORKS.md)
- [services.toml](SERVICES.md)
- [dns.toml](DNS.md)
- [allocations.toml](ALLOCATIONS.md)
- [locations/*.toml](LOCATIONS.md)

## Type notation

| Notation | Meaning |
|---|---|
| `string` | TOML string |
| `bool` | `true` or `false` |
| `u8`, `u16`, `u32` | Non-negative integer within that width |
| `IPv4` | Address such as `10.0.0.10` |
| `IPv4 CIDR` | Network such as `10.0.0.0/24` |
| `IPv6 CIDR` | Network such as `fd00::/48` |
| `array<string>` | TOML string array |
| `map<T>` | TOML table keyed by a user-defined identifier |
