# Proxmox storage adapter

Renders identity-resolved Proxmox storage definitions and `pvesm` reconciliation scripts, currently including NFS-backed storage.

## Inspect and render

```bash
netweft show proxmox-storage --host zion
netweft render proxmox-storage --host zion
```

## Output

```text
~/.local/share/netweft/generated/<location>/hosts/<host>/proxmox-storage/
├── storage.cfg.fragment
├── apply.sh
├── rollback.sh
└── manifest.txt
```

Rendering does not apply the artifact. See the matching deployment guide.
