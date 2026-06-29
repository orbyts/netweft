# Proxmox guest adapter

Renders safe reconciliation for configured VMs and LXCs using `qm` and `pct`, including network attachments, startup settings, passthrough, and backups of guest configuration files.

## Inspect and render

```bash
netweft show guests
netweft render proxmox-guests --host zion
```

## Output

```text
~/.local/share/netweft/generated/<location>/hosts/<host>/proxmox-guests/
├── action-plan.txt
├── apply.sh
├── rollback.sh
└── manifest.txt
```

Rendering does not apply the artifact. See the matching deployment guide.
