# Deployment

Netweft renders artifacts locally. Deployment is explicit.

## Safety classes

### Read-only or manual-plan adapters

- BIND rendering before copy;
- environment files;
- Nginx rendering before deployment;
- Synology NFS permission action plans;
- Proxmox host-network files before manual activation.

### Guarded apply-script adapters

- Docker networking;
- Netplan;
- Proxmox guests;
- Proxmox SDN;
- Proxmox storage;
- systemd network mounts.

Always inspect `action-plan.txt`, generated configuration, and scripts before execution.

## General flow

```text
validate
→ inspect resolved plan
→ render
→ transfer to target host
→ inspect staged artifacts
→ run adapter-specific validation
→ execute apply script or controlled activation
→ verify
→ retain backup path
```

Use [Cross-host transfer](CROSS-HOST-TRANSFER.md) when rendering on another machine.
