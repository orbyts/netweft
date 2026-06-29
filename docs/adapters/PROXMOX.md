# Proxmox host-network adapter

Renders the owned portions of Proxmox ifupdown2 host networking. It preserves `/etc/network/interfaces.d/*` and excludes SDN and corosync state.

## Inspect and render

```bash
netweft show host-network --host zion
netweft render proxmox --host zion
```

## Output

```text
~/.local/share/netweft/generated/<location>/hosts/<host>/proxmox/
├── etc/network/interfaces
├── etc/hosts
├── etc/resolv.conf
└── manifest.txt
```

Rendering does not apply the artifact. See the matching deployment guide.
