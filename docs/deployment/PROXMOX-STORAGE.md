# Deploy Proxmox storage

## Render

```bash
netweft validate
netweft show proxmox-storage --host zion
netweft render proxmox-storage --host zion
```

## Inspect

```bash
cd ~/.local/share/netweft/generated/<location>/hosts/zion/proxmox-storage
cat storage.cfg.fragment
sed -n '1,280p' apply.sh
```

## Apply

```bash
sudo ./apply.sh
```

The script backs up `/etc/pve/storage.cfg`, then uses `pvesm add` or `pvesm set` and reports the resulting storage status.

## Roll back

```bash
sudo ./rollback.sh /root/netweft-backups/proxmox-storage-TIMESTAMP/storage.cfg
```

Confirm NAS-side permissions and network reachability before applying storage definitions.
