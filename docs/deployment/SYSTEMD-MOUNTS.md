# Deploy systemd network mounts

## Render

```bash
netweft validate
netweft show network-mounts --host vortex
netweft render systemd-mounts --host vortex
```

## Inspect and apply

```bash
cd ~/.local/share/netweft/generated/<location>/hosts/vortex/systemd-mounts
find etc/systemd/system -type f -maxdepth 3 -print
sed -n '1,320p' apply.sh
./apply.sh
```

The script:

- backs up replaced units and service drop-ins;
- installs `.mount` units;
- installs `RequiresMountsFor` and `After` drop-ins;
- reloads systemd;
- enables and starts mounts;
- verifies with `findmnt`;
- restarts configured dependent services.

Retain the backup path printed by the script. The current adapter does not emit a standalone rollback script, so restoration is manual from that backup.
