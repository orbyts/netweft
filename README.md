# Netweft

Deterministic network planning and configuration generation for portable infrastructure.

Netweft validates stable infrastructure intent, resolves provider-neutral plans, and renders deterministic artifacts. Deployment remains explicit and reviewable.

## Documentation

- [Documentation index](docs/README.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Getting started](docs/GETTING-STARTED.md)
- [Configuration reference](docs/configuration/README.md)
- [Adapters](docs/adapters/README.md)
- [Deployment](docs/deployment/README.md)
- [Operations](docs/operations/README.md)
- [Development](docs/development/README.md)

## Built-in adapters

```text
bind
nginx
docker
env
netplan
proxmox
proxmox-guests
proxmox-sdn
proxmox-storage
synology-nfs-permissions
systemd-mounts
ssh
cloudflare
```

List the adapters compiled into the current binary:

```bash
netweft adapters list
```

## Common inspection workflow

```bash
netweft validate
netweft show dns
netweft show proxy
netweft show docker --host nexus
netweft show host-network --host zion
netweft show os-network --host quasar
netweft show guests
netweft show proxmox-sdn --host zion
netweft show proxmox-storage --host zion
netweft show network-mounts --host vortex
netweft show nas-permissions
```

Render only after the resolved plan looks correct, then use the matching deployment guide.

## Boundaries

Netweft does not directly:

- copy files over SSH;
- configure routers;
- store secrets;
- issue certificates;
- approve Tailscale routes;
- apply generated artifacts automatically.

Some adapters render guarded `apply.sh`, `verify.sh`, and `rollback.sh` scripts. Running those scripts remains an explicit operator action.

## Development verification

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
git diff --check
```
