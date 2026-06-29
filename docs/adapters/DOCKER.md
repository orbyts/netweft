# Docker networking adapter

Resolves Docker daemon settings and named bridge networks, including IPv4/IPv6 subnets, gateways, trusted host interfaces, and Compose project reconciliation.

## Inspect and render

```bash
netweft show docker --host nexus
netweft render docker --host nexus
```

## Output

```text
~/.local/share/netweft/generated/<location>/hosts/<host>/docker/
├── daemon.desired.json
├── action-plan.txt
├── apply.sh
├── verify.sh
└── rollback.sh
```

Rendering does not apply the artifact. See the matching deployment guide.
