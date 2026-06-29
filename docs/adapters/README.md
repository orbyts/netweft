# Adapters

Adapters translate validated provider-neutral plans into deterministic implementation-specific artifacts.

| Adapter | Purpose | Target scope |
|---|---|---|
| `bind` | BIND authoritative and recursive DNS | location |
| `nginx` | Native reverse-proxy configuration | proxy host |
| `docker` | Docker daemon and named bridge reconciliation | Docker host |
| `env` | Shell and Compose environment files | host |
| `netplan` | Ubuntu Netplan configuration | host |
| `proxmox` | Proxmox ifupdown2 host networking | Proxmox host |
| `proxmox-guests` | VM and LXC reconciliation | Proxmox host |
| `proxmox-sdn` | SDN zones, VNets, and subnets | Proxmox cluster host |
| `proxmox-storage` | Proxmox storage reconciliation | Proxmox host |
| `systemd-mounts` | Network mount units and service dependencies | Linux host |
| `synology-nfs-permissions` | DSM NFS permission action plan | NAS |

```bash
netweft adapters list
```

Adapters render only. A rendered `apply.sh` is an artifact, not an automatic deployment.
