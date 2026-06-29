# Adapter SDK

Adapters consume a validated `ResolvedPlan` and render deterministic artifacts.

Current capability groups include:

```text
DNS
reverse proxy
host environment
Docker networking
host networking
Proxmox guests
Proxmox SDN
Proxmox storage
NAS permissions
network mounts
```

Rules:

- consume resolved plans rather than reparsing TOML;
- require a target host or NAS when the adapter contract needs one;
- own a distinct generated subtree;
- keep secrets outside generated artifacts;
- produce stable manifests;
- never execute deployment during validation or rendering;
- emit guarded apply and rollback scripts where automation is safe enough;
- fail rather than silently guess.
