# systemd network-mount adapter

Renders network mount units and dependent service drop-ins, then enables mounts and restarts configured dependent services.

## Inspect and render

```bash
netweft show network-mounts --host vortex
netweft render systemd-mounts --host vortex
```

## Output

```text
~/.local/share/netweft/generated/<location>/hosts/<host>/systemd-mounts/
├── etc/systemd/system/*.mount
├── etc/systemd/system/<service>.d/10-netweft-network-mounts.conf
├── apply.sh
└── manifest.txt
```

Rendering does not apply the artifact. See the matching deployment guide.
