# Apply Synology NFS permissions

This adapter renders a manual DSM action plan rather than an executable deployment script.

## Render

```bash
netweft validate
netweft show nas-permissions --nas ds1621plus
netweft render synology-nfs-permissions --nas ds1621plus
```

## Apply in DSM

Open:

```text
Control Panel → Shared Folder → select share → Edit → NFS Permissions
```

Follow the generated values in:

```text
~/.local/share/netweft/generated/<location>/nas/<nas>/nfs-permissions/action-plan.txt
```

After applying, test the export from the intended client host before deploying dependent systemd mounts or Proxmox storage.
