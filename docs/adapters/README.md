# Adapters

Adapters translate validated provider-neutral plans into deterministic implementation-specific artifacts.

The output contract is adapter-specific. Rendering does **not** imply that an `install.sh`, `verify.sh`, or `rollback.sh` exists.

| Adapter | Purpose | Typical rendered output | Apply/install | Verify | Rollback |
|---|---|---|---:|---:|---:|
| `bind` | Authoritative and recursive BIND | configuration and zone files | no | no | no |
| `nginx` | Native reverse proxy | `nginx.conf`, `conf.d/*.conf` | no | no | no |
| `env` | Host environment | shell and Compose env files | no | no | no |
| `proxmox` | Proxmox ifupdown2 host networking | `etc/network/interfaces`, `etc/hosts`, `etc/resolv.conf` | no | no | no |
| `synology-nfs-permissions` | DSM NFS permission plan | `action-plan.txt` | manual | manual | manual |
| `docker` | Docker daemon and named bridges | desired state plus guarded scripts | yes | yes | yes |
| `netplan` | Ubuntu Netplan | Netplan payload plus apply/rollback | yes | provider validation | yes |
| `proxmox-guests` | VM and LXC reconciliation | action plan and guarded scripts | yes | adapter-specific | yes |
| `proxmox-sdn` | SDN zones, VNets, subnets | action plan and guarded scripts | yes | yes | yes |
| `proxmox-storage` | Proxmox storage | storage fragment and guarded scripts | yes | adapter-specific | yes |
| `systemd-mounts` | Network mount units | units, drop-ins, apply script | yes | adapter-specific | adapter-specific |
| `ssh` | OpenSSH client includes | config fragments and install/verify/rollback | yes | yes | yes |
| `cloudflare` | Tunnel and DNS reconciliation | API plan, connector bundle, install/verify/rollback | yes | yes | yes |

List adapters compiled into the binary:

```bash
netweft adapters list
```

Always inspect the rendered tree before deployment:

```bash
find ~/.local/share/netweft/generated -type f | sort
```

An apply script is an artifact, not an automatic action. See the matching [deployment guide](../deployment/README.md).
