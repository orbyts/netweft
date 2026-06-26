# Architecture

```text
TOML configuration
    ↓
typed model
    ↓
validation
    ↓
provider-neutral resolution
    ↓
ResolvedPlan
    ↓
adapters
    ├── BIND
    ├── environment
    └── native Nginx
    ↓
deterministic generated artifacts
    ↓
explicit deployment
```

Netweft owns planning and rendering. SSH transport, Docker lifecycle, systemd, certificates, secrets, and router configuration remain outside Netweft.
