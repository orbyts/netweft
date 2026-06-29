# Proxmox SDN adapter

Renders reconciliation for Proxmox SDN zones, VNets, and subnets through `pvesh`, with backup, verification, and rollback artifacts.

## Inspect and render

```bash
netweft show proxmox-sdn --host zion
netweft render proxmox-sdn --host zion
```

## Output

```text
~/.local/share/netweft/generated/<location>/hosts/<host>/proxmox-sdn/
├── action-plan.txt
├── apply.sh
├── verify.sh
├── rollback.sh
└── manifest.txt
```

Rendering does not apply the artifact. See the matching deployment guide.
