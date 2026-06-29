# Netplan adapter

Renders Ubuntu Netplan configuration with guarded `netplan try`, automatic restoration on failure, and a standalone rollback script.

## Inspect and render

```bash
netweft show os-network --host quasar
netweft render netplan --host quasar
```

## Output

```text
~/.local/share/netweft/generated/<location>/hosts/<host>/netplan/
├── etc/netplan/60-netweft.yaml
├── etc/cloud/cloud.cfg.d/99-disable-network-config.cfg
├── apply.sh
├── rollback.sh
└── manifest.txt
```

Rendering does not apply the artifact. See the matching deployment guide.
