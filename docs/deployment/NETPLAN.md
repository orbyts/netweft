# Deploy Netplan

Netplan changes may interrupt the active SSH session. Use console or out-of-band access whenever possible.

## Render

```bash
netweft validate
netweft show os-network --host quasar
netweft render netplan --host quasar
```

## Inspect

```bash
cd ~/.local/share/netweft/generated/<location>/hosts/<host>/netplan
cat etc/netplan/60-netweft.yaml
sed -n '1,260p' apply.sh
```

## Apply

```bash
./apply.sh
```

The generated script:

- backs up `/etc/netplan` and cloud-init network suppression state;
- installs `60-netweft.yaml`;
- runs `netplan generate`;
- uses `netplan try --timeout 60`;
- restores the backup automatically if the guarded apply fails.

## Roll back

```bash
./rollback.sh /var/backups/netweft/netplan-TIMESTAMP
```
