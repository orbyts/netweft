# Netweft

Deterministic network planning and configuration generation for portable infrastructure.

Netweft validates stable infrastructure intent, resolves provider-neutral plans, and renders deterministic artifacts. Deployment remains explicit.

## Documentation

- [Documentation index](docs/README.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Configuration](docs/CONFIGURATION.md)
- [Adapters](docs/adapters/README.md)
- [Deployment](docs/deployment/README.md)
- [Operations](docs/operations/README.md)
- [Development](docs/development/README.md)

## Common workflow

```bash
netweft validate
netweft show dns
netweft show proxy
netweft show env --host nexus
netweft render bind
netweft render env --host nexus
netweft render nginx --host nexus
```

Then use the matching deployment guide.

## Boundaries

Netweft does not directly:

- copy files over SSH;
- restart containers;
- modify routers;
- configure operating-system networking;
- issue certificates;
- store secrets.

## Development verification

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
git diff --check
```
