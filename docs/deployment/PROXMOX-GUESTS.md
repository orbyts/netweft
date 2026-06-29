# Deploy Proxmox guests

## Render

```bash
netweft validate
netweft show guests
netweft render proxmox-guests --host zion
```

## Inspect

```bash
cd ~/.local/share/netweft/generated/<location>/hosts/zion/proxmox-guests
cat action-plan.txt
sed -n '1,320p' apply.sh
```

## Apply

Run on the selected Proxmox host:

```bash
sudo ./apply.sh
```

The script backs up existing LXC and QEMU configuration files, then reconciles configured network and startup properties with `pct set` and `qm set`.

## Roll back

```bash
sudo ./rollback.sh /root/netweft-backups/proxmox-guests-HOST-TIMESTAMP
```

Review passthrough and VirtioFS changes carefully because guest restart requirements are workload-specific.
