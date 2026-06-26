# Deploy environment artifacts

Artifacts are host-specific:

```text
hosts/quasar → Quasar
hosts/nexus  → Nexus
hosts/vortex → Vortex
```

Render:

```bash
netweft validate
netweft show env --host quasar
netweft render env --host quasar
```

If rendered on the target host, no transfer is required.

If rendered elsewhere, stage with SCP and install into the target host's generated tree.

Nexus Compose wrapper:

```text
$DOCKER/services/nexus/compose-netweft
```

It loads:

```text
~/.local/share/netweft/current/hosts/nexus/compose.env
```

Use:

```bash
cd "$DOCKER/services/nexus"
./compose-netweft config
./compose-netweft up -d
```

`docker compose config` may reveal values from local secret `.env` files.
