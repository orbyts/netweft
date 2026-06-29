# Architecture

```text
TOML configuration
    ↓
typed model and validation
    ↓
provider-neutral plans
    ├── DNS
    ├── proxy
    ├── Docker networking
    ├── host networking
    ├── OS networking
    ├── guests
    ├── Proxmox SDN
    ├── Proxmox storage
    ├── network mounts
    └── NAS permissions
    ↓
adapter registry
    ↓
deterministic generated artifacts
    ↓
explicit deployment
```

Netweft separates identity, topology, resolution, rendering, and deployment. The configuration graph is authoritative; adapters do not rediscover or reinterpret TOML independently.

Some renderers generate guarded operational scripts, but Netweft never executes them automatically.
