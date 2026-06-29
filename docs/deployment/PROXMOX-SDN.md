# Deploy Proxmox SDN

## Render

```bash
netweft validate
netweft show proxmox-sdn --host zion
netweft render proxmox-sdn --host zion
```

## Inspect and apply

```bash
cd ~/.local/share/netweft/generated/<location>/hosts/zion/proxmox-sdn
cat action-plan.txt
sed -n '1,320p' apply.sh
sudo ./apply.sh
```

The script backs up `/etc/pve/sdn`, reconciles zones, VNets, and subnets using `pvesh`, and commits SDN state.

## Verify

```bash
sudo ./verify.sh
```

## Roll back

```bash
sudo ./rollback.sh /root/netweft-backups/proxmox-sdn-TIMESTAMP
```

Because `/etc/pve` is managed by pmxcfs, use only the generated rollback script rather than replacing the directory wholesale.
